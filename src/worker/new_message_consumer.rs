use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, BasicPublishArguments, BasicRejectArguments};
use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use async_trait::async_trait;
use diesel::prelude::*;
use serde_json;
use tonic::Status;
use tracing::{debug, error, info, instrument, warn};

use crate::utils::db_connection_manager::DBConnectionManager;
use crate::utils::persistence::message::{InsertMessage, Message};
use crate::utils::persistence::schema::{messages, users_chats};
use crate::utils::persistence::users_chats::UsersChats;
use crate::utils::rabbit_declares::{send_to_error_queue, MESSAGES_EXCHANGE};

#[derive(Clone)]
pub struct NewMessageConsumer {
    connection_manager: Arc<dyn DBConnectionManager>,
}

#[async_trait]
impl AsyncConsumer for NewMessageConsumer {
    #[instrument(skip(self, channel, content))]
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        debug!("Received message");
        if let Err(e) = self.process_message(channel, &deliver, &content).await {
            error!("Failed to process message: {:?}", e);
            if let Err(e) = self.reject_message(channel, &deliver, false, content).await {
                error!("Failed to reject message: {:?}", e);
            };
        }
        debug!("Message processed");
    }
}

impl NewMessageConsumer {
    pub fn new(connection_manager: Arc<dyn DBConnectionManager>) -> Self {
        Self { connection_manager }
    }

    async fn process_message(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        content: &[u8],
    ) -> Result<(), anyhow::Error> {
        let mut db_connection = self.connection_manager.get_connection()?;
        let insert_message = self.deserialize_message(content)?;
        if self
            .check_authority(
                &mut db_connection,
                insert_message.chat_id,
                &insert_message.user_id,
            )
            .await?
        {
            self.insert_and_publish_message(&mut db_connection, channel, &insert_message, deliver)
                .await?;
        }
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .map_err(|e| anyhow::Error::new(e))?;

        Ok(())
    }

    async fn check_authority(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i32,
        user_id: &str,
    ) -> Result<bool, anyhow::Error> {
        let chat = users_chats::table
            .filter(users_chats::chat_id.eq(chat_id))
            .filter(users_chats::user_id.eq(user_id))
            .first::<UsersChats>(db_connection)
            .optional()
            .map_err(|e| {
                error!("Failed to get chat: {}", e);
                Status::internal("Failed to get chat")
            })?;
        Ok(chat.is_some())
    }

    fn deserialize_message(&self, content: &[u8]) -> Result<InsertMessage, serde_json::Error> {
        let message_str = String::from_utf8_lossy(content);
        serde_json::from_str::<InsertMessage>(&message_str)
    }

    async fn insert_and_publish_message(
        &self,
        db_connection: &mut PgConnection,
        channel: &Channel,
        insert_message: &InsertMessage,
        deliver: &Deliver,
    ) -> Result<(), anyhow::Error> {
        let message = self.insert_message(db_connection, insert_message)?;
        self.publish_message(channel, &message, deliver).await?;
        Ok(())
    }

    fn insert_message(
        &self,
        db_connection: &mut PgConnection,
        insert_message: &InsertMessage,
    ) -> Result<Message, diesel::result::Error> {
        diesel::insert_into(messages::table)
            .values(insert_message)
            .get_result(db_connection)
    }

    async fn publish_message(
        &self,
        channel: &Channel,
        message: &Message,
        deliver: &Deliver,
    ) -> Result<(), anyhow::Error> {
        let serialized_message = serde_json::to_string(message)?;
        let exchange_name = format!("{}-{}", MESSAGES_EXCHANGE, message.chat_id);
        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(&exchange_name, "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| anyhow::Error::new(e))?;

        debug!("Message published successfully");
        Ok(())
    }

    #[instrument(skip(self, channel))]
    async fn reject_message(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        requeue: bool,
        content: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        warn!("Rejecting message");
        send_to_error_queue(channel, content).await?;

        channel
            .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), requeue))
            .await
            .map_err(|e| {
                error!("Failed to reject message: {:?}", e);
                anyhow::Error::new(e)
            })
    }
}
