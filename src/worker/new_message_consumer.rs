use std::sync::Arc;

use amqprs::channel::{
    BasicPublishArguments, BasicRejectArguments, ExchangeDeclareArguments, ExchangeType,
};
use amqprs::{
    channel::{BasicAckArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver, FieldTable,
};
use async_trait::async_trait;
use diesel::prelude::*;
use serde_json;
use shaku::Component;
use tracing::{error, info};

use crate::utils::db_connection_manager::DBConnectionManager;
use crate::utils::persistence::message::{InsertMessage, Message};
use crate::utils::persistence::schema::messages;
use crate::utils::rabbit_declares::{declare_messages_exchange, MESSAGES_EXCHANGE};

#[derive(Clone)]
pub struct NewMessageConsumer {
    connection_manager: Arc<dyn DBConnectionManager>,
}

impl NewMessageConsumer {
    pub fn new(connection_manager: Arc<dyn DBConnectionManager>) -> Self {
        Self { connection_manager }
    }
}

#[async_trait]
impl AsyncConsumer for NewMessageConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        let mut db_connection = self
            .connection_manager
            .get_connection()
            .expect("Couldn't get DB connection");

        let message_str = String::from_utf8_lossy(&content);
        match serde_json::from_str::<InsertMessage>(&message_str) {
            Ok(mut insert_message) => {
                insert_message.user_id = "google-oauth2|108706181521622783833".to_string();

                match diesel::insert_into(messages::table)
                    .values(&insert_message)
                    .get_result::<Message>(&mut db_connection)
                {
                    Ok(message) => {
                        // Successfully inserted message
                        // You can use `message` here, which contains the inserted data including id and timestamp
                        info!("Exchange declared successfully");
                        match serde_json::to_string(&message) {
                            Ok(serialized_message) => {
                                // Publish the message
                                match channel
                                    .basic_publish(
                                        BasicProperties::default(),
                                        serialized_message.into_bytes(),
                                        BasicPublishArguments::new(
                                            MESSAGES_EXCHANGE,
                                            &format!("{}", message.chat_id),
                                        )
                                        .mandatory(false)
                                        .immediate(false)
                                        .finish(),
                                    )
                                    .await
                                {
                                    Ok(_) => info!("Message published successfully"),
                                    Err(e) => error!("Failed to publish message: {:?}", e),
                                }
                            }
                            Err(e) => error!("Failed to serialize message: {:?}", e),
                        }

                        // Serialize the message to a JSON string
                        let serialized_message = serde_json::to_string(&message);

                        if let Err(e) = channel
                            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
                            .await
                        {
                            error!("Error acknowledging message: {:?}", e);
                        }
                    }
                    Err(e) => {
                        // Handle error in insertion
                        error!("Error inserting message into database: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to deserialize message: {:?}", e);
                if let Err(e) = channel
                    .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), false))
                    .await
                {
                    error!("Error acknowledging message: {:?}", e);
                }
            }
        }
    }
}
