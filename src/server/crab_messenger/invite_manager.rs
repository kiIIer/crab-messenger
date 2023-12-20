use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments,
    QueueDeclareArguments,
};
use amqprs::BasicProperties;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::server::crab_messenger::invite_manager::invite_consumer::RabbitConsumer;
use crate::server::crab_messenger::InviteResponseStream;
use crate::utils::generate_random_string;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::{module, Component, Interface};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::field::debug;
use tracing::{debug, error, info};

use crate::utils::messenger::{
    Invite as ProtoInvite, InvitesRequest, SendInviteRequest, SendInviteResponse,
};
use crate::utils::persistence::schema::messages::user_id;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{
    declare_invites_exchange, declare_send_invite_exchange, INVITES_EXCHANGE, SEND_INVITE_EXCHANGE,
};

mod invite_consumer;

#[async_trait]
pub trait InviteManager: Interface {
    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status>;

    async fn invites(
        &self,
        request: Request<InvitesRequest>,
    ) -> Result<Response<InviteResponseStream>, Status>;
}

#[derive(Component, Clone)]
#[shaku(interface = InviteManager)]
pub struct InviteManagerImpl {
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[derive(Serialize, Deserialize)]
struct RabbitCreateInvite {
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub chat_id: i32,
}

#[async_trait]
impl InviteManager for InviteManagerImpl {
    #[tracing::instrument(skip(self, request), err)]
    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status> {
        info!("Sending invite");

        let metadata = request.metadata();

        let inviter_user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let invite_request = request.into_inner();
        let rabbit_invite = RabbitCreateInvite {
            inviter_user_id,
            invitee_user_id: invite_request.user_id,
            chat_id: invite_request.chat_id,
        };

        let serialized_message = serde_json::to_string(&rabbit_invite).map_err(|e| {
            error!("Failed to serialize message: {:?}", e);
            Status::internal("Failed to serialize message")
        })?;

        let channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            Status::internal("Failed to get channel")
        })?;

        declare_send_invite_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            Status::internal("Failed to declare exchange")
        })?;

        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(SEND_INVITE_EXCHANGE, "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish message: {:?}", e);
                Status::internal("Failed to publish message")
            })?;

        Ok(Response::new(SendInviteResponse { success: true }))
    }

    #[tracing::instrument(skip(self, request), err)]
    async fn invites(
        &self,
        request: Request<InvitesRequest>,
    ) -> Result<Response<InviteResponseStream>, Status> {
        info!("Starting invites");
        let metadata = request.metadata();
        let listener_user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let (tx, rx) = mpsc::channel(16);
        let channel = self.setup_invite_channel().await?;

        let queue_name = generate_random_string(16);
        let routing_key = &listener_user_id;

        debug!("Setting up queue");
        self.setup_queue(&channel, &queue_name, &routing_key)
            .await?;

        let consumer = RabbitConsumer::new(tx.clone(), queue_name.clone());
        debug!("Consuming messages");
        let self_clone = self.clone();
        let consumer_channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            Status::internal("Failed to get channel")
        })?;

        let res = self_clone
            .consume_messages(consumer_channel, consumer, &queue_name)
            .await
            .map_err(|e| {
                error!("Failed to consume messages: {:?}", e);
                Status::internal("Failed to consume messages")
            })?;
        debug!("Consume messages result: {:?}", res);

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

impl InviteManagerImpl {
    #[tracing::instrument(skip(self))]
    async fn setup_invite_channel(&self) -> Result<Channel, Status> {
        let channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            Status::internal("Failed to get channel")
        })?;

        declare_invites_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            Status::internal("Failed to declare exchange")
        })?;

        Ok(channel)
    }

    #[tracing::instrument(skip(self, channel))]
    async fn setup_queue(
        &self,
        channel: &Channel,
        queue_name: &str,
        routing_key: &str,
    ) -> Result<(), Status> {
        channel
            .queue_declare(
                QueueDeclareArguments::new(queue_name)
                    .auto_delete(true)
                    .durable(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to declare queue: {:?}", e);
                Status::internal("Failed to declare queue")
            })?;

        channel
            .queue_bind(QueueBindArguments::new(queue_name, INVITES_EXCHANGE, routing_key).finish())
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                Status::internal("Failed to bind queue")
            })?;

        Ok(())
    }

    #[tracing::instrument(skip(self, channel, consumer))]
    async fn consume_messages(
        &self,
        channel: Channel,
        mut consumer: RabbitConsumer,
        queue_name: &str,
    ) -> anyhow::Result<()> {
        let (tag, message_rx) = channel
            .basic_consume_rx(BasicConsumeArguments::new(queue_name, "").finish())
            .await
            .map_err(|e| {
                error!("Failed to consume: {:?}", e);
                Status::internal("Failed to consume messages")
            })?;

        let handle = tokio::spawn(async move {
            consumer.consume(&channel, tag, message_rx).await;
        });

        Ok(())
    }
}

module! {
    pub InviteManagerModule {
        components = [InviteManagerImpl],
        providers = [],
        use ChannelManagerModule {
            components = [dyn ChannelManager],
            providers = []
        }
    }
}

pub fn build_invite_manager_module() -> Arc<InviteManagerModule> {
    Arc::new(InviteManagerModule::builder(build_channel_manager_module()).build())
}
