use std::sync::Arc;

use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, QueueBindArguments, QueueDeclareArguments,
};
use amqprs::{BasicProperties, DELIVERY_MODE_PERSISTENT};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use futures_core::Stream;
use rand::Rng;
use shaku::{module, Component, Interface};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream;
use tonic::{Request, Response, Status, Streaming};

use crate::server::crab_messenger::ResponseStream;
use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::generate_random_string;
use crate::utils::messenger::messenger_server::Messenger;
use crate::utils::messenger::{GetMessages, Messages, SendMessage};
use crate::utils::persistence::message::{InsertMessage, Message};
use crate::utils::persistence::schema::messages;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{
    declare_messages_exchange, declare_new_message_exchange, MESSAGES_EXCHANGE,
    NEW_MESSAGE_EXCHANGE,
};

mod message_consumer;

#[async_trait]
pub trait ChatManager: Interface {
    type ChatStream;
    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status>;
    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status>;
}

#[derive(Component)]
#[shaku(interface = ChatManager<ChatStream = ResponseStream>)]
pub struct ChatManagerImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[async_trait]
impl ChatManager for ChatManagerImpl {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        println!("Connecting client");
        let (tx, rx) = mpsc::channel(16);
        let channel = self.channel_manager.get_channel().await.unwrap();

        let queue_name = generate_random_string(10);
        let routing_key = "1";

        declare_new_message_exchange(&channel)
            .await
            .expect("couldn't create exchange new messages");

        declare_messages_exchange(&channel)
            .await
            .expect("Could 't create exchange for just messages");

        channel
            .queue_declare(QueueDeclareArguments::new(&queue_name))
            .await
            .expect("Couldn't declare queue for messages");
        channel
            .queue_bind(QueueBindArguments::new(
                &queue_name,
                MESSAGES_EXCHANGE,
                routing_key,
            ))
            .await
            .expect("Couldn't bind queue to exchange");

        let consumer = message_consumer::RabbitConsumer::new(tx.clone());
        channel
            .basic_consume(consumer, BasicConsumeArguments::new(&queue_name, ""))
            .await
            .unwrap();

        tokio::spawn(async move {
            let mut stream = request.into_inner();

            while let Ok(send_msg_result) = stream.message().await {
                println!("We got a message!");
                match send_msg_result {
                    Some(send_msg) => {
                        // Convert `SendMessage` to `InsertMessage`
                        let insert_message = InsertMessage {
                            user_id: "google-oauth2|108706181521622783833".to_string(),
                            text: send_msg.text,
                            chat_id: send_msg.chat_id,
                        };

                        // Serialize `InsertMessage` to JSON
                        match serde_json::to_string(&insert_message) {
                            Ok(serialized_message) => {
                                // Publish to RabbitMQ
                                let exchange_name = NEW_MESSAGE_EXCHANGE;
                                let basic_properties = BasicProperties::default()
                                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                                    .finish(); // Persistent message

                                match channel
                                    .basic_publish(
                                        basic_properties,
                                        serialized_message.into_bytes(),
                                        BasicPublishArguments::new(exchange_name, "")
                                            .mandatory(false)
                                            .immediate(false)
                                            .finish(),
                                    )
                                    .await
                                {
                                    Ok(_) => println!("Message published to NewMessageExchange"),
                                    Err(e) => eprintln!("Failed to publish message: {:?}", e),
                                }
                            }
                            Err(e) => eprintln!("Failed to serialize message: {:?}", e),
                        }
                    }
                    None => {
                        eprintln!("Stream ended or error in receiving message from stream");
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
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

        let message_results = messages::table
            .filter(messages::chat_id.eq(chat_id_filter))
            .filter(messages::created_at.lt(created_before_naive))
            .load::<Message>(&mut connection)
            .map_err(|e| Status::internal(format!("Failed to query messages: {}", e)))?;

        let proto_messages: Vec<_> = message_results.into_iter().map(Into::into).collect();

        let response = Messages {
            messages: proto_messages,
        };

        Ok(Response::new(response))
    }
}

module! {
    pub ChatManagerModule {
        components = [ChatManagerImpl],
        providers = [],
         use DBConnectionManagerModule{
            components = [dyn DBConnectionManager],
            providers = [],
        },
        use ChannelManagerModule{
            components = [dyn ChannelManager],
            providers = [],
        },
    }
}

pub fn build_chat_manager_module() -> Arc<ChatManagerModule> {
    Arc::new(
        ChatManagerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
        )
        .build(),
    )
}
