use std::sync::Arc;

use amqprs::channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use shaku::{module, Component, Interface};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info};

use crate::server::crab_messenger::message_manager::message_consumer::RabbitConsumer;
use crate::server::crab_messenger::message_manager::message_stream_handler::{
    build_message_stream_handler_module, MessageStreamHandler, MessageStreamHandlerModule,
};
use crate::server::crab_messenger::ChatResponseStream;
use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::generate_random_string;
use crate::utils::messenger::{GetMessagesRequest, Messages, SendMessage};
use crate::utils::persistence::message::Message;
use crate::utils::persistence::schema::messages;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{declare_new_message_exchange, MESSAGES_EXCHANGE};

mod message_consumer;
mod message_stream_handler;

#[async_trait]
pub trait MessageManager: Interface {
    type ChatStream;
    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status>;
    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<Messages>, Status>;
}

#[derive(Component)]
#[shaku(interface = MessageManager<ChatStream = ChatResponseStream>)]
pub struct MessageManagerImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
    #[shaku(inject)]
    message_stream_handler: Arc<dyn MessageStreamHandler>,
}

impl MessageManagerImpl {
    async fn setup_chat_channel(&self) -> Result<Channel, Status> {
        self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            Status::internal("Failed to get channel")
        })
    }

    async fn setup_queue(
        &self,
        channel: &Channel,
        queue_name: &str,
        routing_key: &str,
    ) -> Result<(), Status> {
        declare_new_message_exchange(channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            Status::internal("Failed to declare exchange")
        })?;

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
            .queue_bind(
                QueueBindArguments::new(queue_name, MESSAGES_EXCHANGE, routing_key).finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                Status::internal("Failed to bind queue")
            })?;

        Ok(())
    }

    async fn consume_messages(
        &self,
        channel: &Channel,
        consumer: RabbitConsumer,
        queue_name: &str,
    ) -> Result<String, Status> {
        channel
            .basic_consume(
                consumer,
                BasicConsumeArguments::new(queue_name, "").finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to consume: {:?}", e);
                Status::internal("Failed to consume messages")
            })
    }
}

#[async_trait]
impl MessageManager for MessageManagerImpl {
    type ChatStream = ChatResponseStream;

    #[tracing::instrument(skip(self, request))]
    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        info!("Starting chat");
        let metadata = request.metadata();
        debug!("User_id: {:?}", metadata.get("user_id"));
        let (tx, rx) = mpsc::channel(16);
        let channel = self.setup_chat_channel().await?;

        let queue_name = generate_random_string(16);
        let routing_key = "1"; // Adjust the routing key logic as needed

        self.setup_queue(&channel, &queue_name, routing_key).await?;

        let consumer = RabbitConsumer::new(tx.clone(), queue_name.clone());
        self.consume_messages(&channel, consumer, &queue_name)
            .await?;

        let message_stream_handler = self.message_stream_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = message_stream_handler
                .handle_stream(request.into_inner(), &channel)
                .await
            {
                error!("Error handling stream: {:?}", e);
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<Messages>, Status> {
        info!("Received request to get messages");

        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection from pool: {}", e);
            Status::internal(format!("Failed to get DB connection from pool: {}", e))
        })?;

        let get_messages_req = request.into_inner();
        let chat_id_filter = get_messages_req.chat_id;

        let created_before_filter = get_messages_req.created_before.unwrap(); // assuming it's present
        let created_before_naive = chrono::NaiveDateTime::from_timestamp_opt(
            created_before_filter.seconds,
            created_before_filter.nanos as u32,
        )
        .unwrap();
        debug!(
            "Fetching messages for chat_id: {} created before: {:?}",
            chat_id_filter, created_before_naive
        );

        let message_results = match messages::table
            .filter(messages::chat_id.eq(chat_id_filter))
            .filter(messages::created_at.lt(created_before_naive))
            .load::<Message>(&mut connection)
        {
            Ok(results) => {
                debug!("Successfully queried messages from database");
                results
            }
            Err(e) => {
                error!("Failed to query messages: {}", e);
                return Err(Status::internal(format!("Failed to query messages: {}", e)));
            }
        };

        let proto_messages: Vec<_> = message_results.into_iter().map(Into::into).collect();
        debug!("Total messages fetched: {}", proto_messages.len());

        let response = Messages {
            messages: proto_messages,
        };

        info!("Successfully processed get_messages request");
        Ok(Response::new(response))
    }
}

module! {
    pub MessageManagerModule {
        components = [MessageManagerImpl],
        providers = [],
         use DBConnectionManagerModule{
            components = [dyn DBConnectionManager],
            providers = [],
        },
        use ChannelManagerModule{
            components = [dyn ChannelManager],
            providers = [],
        },
        use MessageStreamHandlerModule{
            components = [dyn MessageStreamHandler],
            providers = [],
        },
    }
}

pub fn build_message_manager_module() -> Arc<MessageManagerModule> {
    Arc::new(
        MessageManagerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
            build_message_stream_handler_module(),
        )
        .build(),
    )
}
