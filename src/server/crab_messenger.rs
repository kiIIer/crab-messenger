use crate::server::crab_messenger::message_consumer::RabbitConsumer;
use crate::server::persistence::message::{InsertMessage, Message};
use crate::server::persistence::schema::messages;
use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::messenger::messenger_server::Messenger;
use crate::utils::messenger::{GetMessages, Message as MMessage, Messages, SendMessage};
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, ExchangeDeclareArguments, ExchangeType,
    QueueBindArguments, QueueDeclareArguments,
};
use amqprs::{BasicProperties, FieldTable, DELIVERY_MODE_PERSISTENT};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{PgConnection, RunQueryDsl};
use dotenv::dotenv;
use futures_core::Stream;
use shaku::{module, Component, Interface};
use std::env;
use std::pin::Pin;
use std::sync::Arc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::error;

mod message_consumer;

pub trait CrabMessenger: Interface + Messenger {}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<MMessage, Status>> + Send>>;

#[derive(Component)]
#[shaku(interface = CrabMessenger<ChatStream = ResponseStream>)]
pub struct CrabMessengerImpl {
    #[shaku(inject)]
    connection_manager: Arc<dyn DBConnectionManager>,
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

impl CrabMessenger for CrabMessengerImpl {}

#[async_trait]
impl Messenger for CrabMessengerImpl {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        println!("Connecting client");
        let (tx, rx) = mpsc::channel(16);
        let channel = self.channel_manager.get_channel().await.unwrap();

        let queue_name = generate_random_string(10);
        let exchange_name = "MessagesExchange";
        let routing_key = "1";

        channel
            .exchange_declare(ExchangeDeclareArguments::new(
                "NewMessageExchange",
                "direct",
            ))
            .await
            .expect("couldn't create exchange new messages");
        channel
            .exchange_declare(
                ExchangeDeclareArguments::of_type(exchange_name, ExchangeType::Fanout)
                    .passive(false)
                    .durable(false)
                    .auto_delete(false)
                    .internal(false)
                    .no_wait(false)
                    .arguments(FieldTable::default())
                    .finish(),
            )
            .await
            .expect("Could 't create exhange for just messages");
        channel
            .queue_declare(QueueDeclareArguments::new(&queue_name))
            .await
            .unwrap();
        channel
            .queue_bind(QueueBindArguments::new(
                &queue_name,
                exchange_name,
                routing_key,
            ))
            .await
            .unwrap(); // Bind to the exchange with routing key "1"

        let consumer = RabbitConsumer::new(tx.clone());
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
                                let exchange_name = "NewMessageExchange"; // The name of your exchange for persistent messages
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
        let mut connection = self.connection_manager.get_connection().map_err(|e| {
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

pub struct MessengerAdapter {
    messenger: Arc<dyn CrabMessenger<ChatStream = ResponseStream>>,
}

impl MessengerAdapter {
    pub fn new(messenger: Arc<dyn CrabMessenger<ChatStream = ResponseStream>>) -> Self {
        Self { messenger }
    }
}

#[async_trait]
impl Messenger for MessengerAdapter {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        self.messenger.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        self.messenger.get_messages(request).await
    }
}

fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}


module! {
    pub CrabMessengerModule{
        components = [CrabMessengerImpl],
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

pub fn build_crab_messenger_module() -> Arc<CrabMessengerModule> {
    Arc::new(
        CrabMessengerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
        )
        .build(),
    )
}
