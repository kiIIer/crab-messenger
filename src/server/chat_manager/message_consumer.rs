use crate::utils::persistence::message::Message as DBMessage;
use crate::utils::messenger::Message as GMessage;
use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tonic::Status;

pub struct RabbitConsumer {
    tx: mpsc::Sender<Result<GMessage, Status>>,
}

impl RabbitConsumer {
    pub fn new(tx: mpsc::Sender<Result<GMessage, Status>>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl AsyncConsumer for RabbitConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        // Deserialize the DB message
        let db_message: DBMessage = serde_json::from_slice(&content).unwrap(); // Handle errors properly

        // Convert to gRPC message
        let grpc_message = GMessage {
            id: db_message.id,
            user_id: db_message.user_id,
            chat_id: db_message.chat_id,
            text: db_message.text,
            created_at: Some(Timestamp {
                seconds: db_message.created_at.timestamp(),
                nanos: db_message.created_at.timestamp_subsec_nanos() as i32,
            }),
        };

        // Send to client
        let _ = self.tx.send(Ok(grpc_message)).await;
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}
