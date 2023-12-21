use std::sync::Arc;

use amqprs::channel::{BasicPublishArguments, Channel};
use amqprs::BasicProperties;
use async_trait::async_trait;
use shaku::{module, Component, Interface};
use tonic::Streaming;
use tracing::{debug, error, info};

use crate::utils::messenger::SendMessage;
use crate::utils::persistence::message::InsertMessage;
use crate::utils::rabbit_declares::NEW_MESSAGE_EXCHANGE;

#[async_trait]
pub trait MessageStreamHandler: Interface {
    async fn handle_stream(
        &self,
        stream: Streaming<SendMessage>,
        channel: &Channel,
        user_id: String,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Component)]
#[shaku(interface = MessageStreamHandler)]
pub struct MessageStreamHandlerImpl;

impl MessageStreamHandlerImpl {
    async fn publish_message(
        &self,
        channel: &Channel,
        serialized_message: String,
    ) -> Result<(), anyhow::Error> {
        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(NEW_MESSAGE_EXCHANGE, "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish message: {:?}", e);
                anyhow::Error::new(e)
            })?;

        Ok(())
    }
}

#[async_trait]
impl MessageStreamHandler for MessageStreamHandlerImpl {
    #[tracing::instrument(skip(self, stream, channel))]
    async fn handle_stream(
        &self,
        mut stream: Streaming<SendMessage>,
        channel: &Channel,
        user_id: String,
    ) -> Result<(), anyhow::Error> {
        loop {
            let message_result = stream.message().await;
            match message_result {
                Ok(Some(send_msg)) => {
                    let insert_message = InsertMessage {
                        user_id: user_id.clone(),
                        text: send_msg.text,
                        chat_id: send_msg.chat_id,
                    };

                    let serialized_message =
                        serde_json::to_string(&insert_message).map_err(anyhow::Error::new)?;

                    self.publish_message(channel, serialized_message).await?;
                    debug!("Message published successfully");
                }
                Ok(None) => {
                    info!("Stream closed by sender");
                    break;
                }
                Err(e) => {
                    error!("Stream error: {:?}", e);
                    return Err(anyhow::Error::new(e));
                }
            }
        }
        Ok(())
    }
}

module! {
    pub MessageStreamHandlerModule {
        components = [MessageStreamHandlerImpl],
        providers = [],
    }
}

pub fn build_message_stream_handler_module() -> Arc<MessageStreamHandlerModule> {
    Arc::new(MessageStreamHandlerModule::builder().build())
}
