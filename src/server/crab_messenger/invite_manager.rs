use std::sync::Arc;

use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments,
    QueueDeclareArguments,
};
use amqprs::BasicProperties;
use async_trait::async_trait;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{QueryDsl, RunQueryDsl};
use r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use shaku::{module, Component, Interface};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::server::crab_messenger::invite_manager::invite_consumer::RabbitConsumer;
use crate::server::crab_messenger::InviteResponseStream;
use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::generate_random_string;
use crate::utils::messenger::{
    AnswerInviteRequest, AnswerInviteResponse, GetInvitesRequest, GetInvitesResponse,
    InvitesRequest, SendInviteRequest, SendInviteResponse,
};
use crate::utils::persistence::invite::Invite;
use crate::utils::persistence::schema::invites;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{
    declare_accept_invites_exchange, declare_invites_exchange, declare_send_invite_exchange,
    ACCEPT_INVITES_EXCHANGE, INVITES_EXCHANGE, SEND_INVITE_EXCHANGE,
};
use crate::utils::rabbit_types::RabbitInviteAccept;

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

    async fn get_invites(
        &self,
        request: Request<GetInvitesRequest>,
    ) -> Result<Response<GetInvitesResponse>, Status>;

    async fn answer_invite(
        &self,
        request: Request<AnswerInviteRequest>,
    ) -> Result<Response<AnswerInviteResponse>, Status>;
}

#[derive(Component, Clone)]
#[shaku(interface = InviteManager)]
pub struct InviteManagerImpl {
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
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

    async fn get_invites(
        &self,
        request: Request<GetInvitesRequest>,
    ) -> Result<Response<GetInvitesResponse>, Status> {
        let metadata = request.metadata();
        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get connection: {:?}", e);
            Status::internal("Failed to get connection")
        })?;

        let invites = invites::table
            .filter(invites::invitee_user_id.eq(user_id))
            .select(invites::all_columns)
            .load::<Invite>(&mut connection)
            .map_err(|e| {
                error!("Failed to get chats: {}", e);
                Status::internal("Failed to get chats")
            })?;

        Ok(Response::new(GetInvitesResponse {
            invites: invites.into_iter().map(|i| i.into()).collect(),
        }))
    }

    async fn answer_invite(
        &self,
        request: Request<AnswerInviteRequest>,
    ) -> Result<Response<AnswerInviteResponse>, Status> {
        let metadata = request.metadata();
        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let answer_invite_request = request.into_inner();

        match answer_invite_request.accept {
            false => self
                .answer_nay(
                    &mut self.db_connection_manager.get_connection().map_err(|e| {
                        error!("Failed to get connection: {:?}", e);
                        Status::internal("Failed to get connection")
                    })?,
                    answer_invite_request.invite_id,
                    &user_id,
                )
                .await
                .map_err(|e| {
                    error!("Failed to answer invite: {:?}", e);
                    Status::internal("Failed to answer invite")
                })?,
            true => self
                .answer_yay(
                    &self.channel_manager.get_channel().await.map_err(|e| {
                        error!("Failed to get channel: {:?}", e);
                        Status::internal("Failed to get channel")
                    })?,
                    answer_invite_request.invite_id,
                    &user_id,
                )
                .await
                .map_err(|e| {
                    error!("Failed to answer invite: {:?}", e);
                    Status::internal("Failed to answer invite")
                })?,
        }

        Ok(Response::new(AnswerInviteResponse { success: true }))
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

    async fn answer_yay(
        &self,
        channel: &Channel,
        invite_id: i32,
        user_id: &str,
    ) -> anyhow::Result<()> {
        info!("Answering yay to invite {}", invite_id);
        declare_accept_invites_exchange(&channel)
            .await
            .map_err(|e| {
                error!("Failed to declare exchange: {:?}", e);
                Status::internal("Failed to declare exchange")
            })?;
        let rabbit_invite_accept = RabbitInviteAccept {
            invite_id,
            user_id: user_id.to_string(),
        };

        let serialized_message = serde_json::to_string(&rabbit_invite_accept).map_err(|e| {
            error!("Failed to serialize message: {:?}", e);
            Status::internal("Failed to serialize message")
        })?;

        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(ACCEPT_INVITES_EXCHANGE, "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish message: {:?}", e);
                Status::internal("Failed to publish message")
            })?;

        Ok(())
    }

    async fn answer_nay(
        &self,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        invite_id: i32,
        user_id: &str,
    ) -> anyhow::Result<()> {
        info!("Answering nay to invite {}", invite_id);
        diesel::delete(invites::table)
            .filter(invites::id.eq(invite_id))
            .filter(invites::invitee_user_id.eq(user_id))
            .execute(connection)
            .map_err(|e| {
                error!("Failed to delete invite: {:?}", e);
                Status::internal("Failed to delete invite")
            })?;

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
        },
        use DBConnectionManagerModule {
            components = [dyn DBConnectionManager],
            providers = []
        }
    }
}

pub fn build_invite_manager_module() -> Arc<InviteManagerModule> {
    Arc::new(
        InviteManagerModule::builder(
            build_channel_manager_module(),
            build_db_connection_manager_module(),
        )
        .build(),
    )
}
