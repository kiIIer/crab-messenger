use amqprs::channel::{BasicAckArguments, Channel, QueueDeleteArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use scopeguard::defer;
use tokio::sync::mpsc;
use tonic::Status;
use tracing::{debug, error, info};

use crate::utils::messenger::Message as GMessage;
use crate::utils::persistence::message::Message as DBMessage;

pub struct RabbitConsumer {
    tx: mpsc::Sender<Result<GMessage, Status>>,
    queue_name: String,
}

impl RabbitConsumer {
    pub fn new(tx: mpsc::Sender<Result<GMessage, Status>>, queue_name: String) -> Self {
        Self { tx, queue_name }
    }
}

#[async_trait]
impl AsyncConsumer for RabbitConsumer {
    #[tracing::instrument(skip(self, channel, content))]
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        debug!("Sending message to user");
        let db_message: DBMessage = match serde_json::from_slice(&content) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to deserialize message: {:?}", e);
                return;
            }
        };

        let grpc_message = db_message.into();

        let send_result = self.tx.send(Ok(grpc_message)).await;
        debug!("Send result: {:?}", send_result);

        if let Err(e) = channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
        {
            error!("Failed to acknowledge message: {:?}", e);
        }
    }
}
