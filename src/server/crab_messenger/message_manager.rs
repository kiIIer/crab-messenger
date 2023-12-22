use std::sync::Arc;

use amqprs::channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments};
use amqprs::consumer::AsyncConsumer;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use shaku::{module, Component, Interface};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::Body;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info};

use crate::server::crab_messenger::message_manager::connect_consumer::ConnectConsumer;
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
use crate::utils::persistence::schema::{messages, users_chats};
use crate::utils::persistence::users_chats::UsersChats;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{
    chat_connect_exchange_name, declare_chat_connect_exchange, declare_messages_exchange,
    declare_new_message_exchange, messages_exchange_name, CHAT_CONNECT_EXCHANGE, MESSAGES_EXCHANGE,
};

mod connect_consumer;
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

    async fn setup_messages_queue(
        &self,
        channel: &Channel,
        queue_name: &str,
        chat_id: &str,
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

        let exchange_name = messages_exchange_name(&chat_id);
        declare_messages_exchange(channel, chat_id)
            .await
            .map_err(|e| {
                error!("Failed to declare exchange: {:?}", e);
                Status::internal("Failed to declare exchange")
            })?;
        channel
            .queue_bind(QueueBindArguments::new(queue_name, &exchange_name, "").finish())
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                Status::internal("Failed to bind queue")
            })?;

        Ok(())
    }

    async fn setup_connect_queue(
        &self,
        channel: &Channel,
        queue_name: &str,
        user_id: &str,
    ) -> Result<(), Status> {
        declare_chat_connect_exchange(channel, user_id)
            .await
            .map_err(|e| {
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
                QueueBindArguments::new(queue_name, &chat_connect_exchange_name(user_id), "")
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                Status::internal("Failed to bind queue")
            })?;

        Ok(())
    }

    async fn consume_messages<T>(
        &self,
        channel: &Channel,
        consumer: T,
        queue_name: &str,
    ) -> Result<String, Status>
    where
        T: AsyncConsumer + Send + 'static,
    {
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
        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let user_id_clone = user_id.clone();
        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection: {}", e);
            Status::internal("Failed to get DB connection")
        })?;
        let (tx, rx) = mpsc::channel(16);
        let channel = self.setup_chat_channel().await.map_err(|e| {
            error!("Failed to setup chat channel: {:?}", e);
            Status::internal("Failed to setup chat channel")
        })?;

        let queue_name = generate_random_string(16);

        let connect_queue_name = format!("connect-{}", &queue_name);
        self.setup_connect_queue(&channel, &connect_queue_name, &user_id)
            .await
            .map_err(|e| {
                error!("Failed to setup connect queue: {:?}", e);
                Status::internal("Failed to setup connect queue")
            })?;

        let connect_consumer = ConnectConsumer::new(queue_name.clone());
        let connect_consumer_tag = self
            .consume_messages(&channel, connect_consumer, &connect_queue_name)
            .await?;

        let my_chats = users_chats::table
            .filter(users_chats::user_id.eq(user_id))
            .select(users_chats::all_columns)
            .load::<UsersChats>(&mut connection)
            .map_err(|e| {
                error!("Failed to get chats: {}", e);
                Status::internal("Failed to get chats")
            })?;

        for chat in my_chats {
            let chat_id = format!("{}", chat.chat_id);
            self.setup_messages_queue(&channel, &queue_name, &chat_id)
                .await?;
        }

        let cunsumer_tag = self
            .consume_messages(
                &channel,
                RabbitConsumer::new(tx, queue_name.clone()),
                &queue_name,
            )
            .await?;

        let message_stream_handler = self.message_stream_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = message_stream_handler
                .handle_stream(request.into_inner(), &channel, user_id_clone)
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
        let metadata = request.metadata();
        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection from pool: {}", e);
            Status::internal(format!("Failed to get DB connection from pool: {}", e))
        })?;

        let get_messages_req = request.into_inner();
        let chat_id_filter = get_messages_req.chat_id;

        let binding = users_chats::table
            .filter(users_chats::user_id.eq(user_id))
            .filter(users_chats::chat_id.eq(chat_id_filter))
            .first::<UsersChats>(&mut connection)
            .optional()
            .map_err(|e| {
                error!("Failed to get binding: {}", e);
                Status::internal(format!("Failed to get binding: {}", e))
            })?;

        debug!("Binding: {:?}", binding);

        if binding.is_none() {
            return Err(Status::not_found(format!(
                "No binding found for chat_id: {}",
                chat_id_filter
            )));
        }

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
