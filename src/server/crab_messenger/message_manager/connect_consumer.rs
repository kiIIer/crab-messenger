use crate::utils::rabbit_declares::{declare_messages_exchange, MESSAGES_EXCHANGE};
use amqprs::channel::{BasicAckArguments, BasicRejectArguments, Channel, QueueBindArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use tracing::error;

pub struct ConnectConsumer {
    queue_name: String,
}

impl ConnectConsumer {
    pub fn new(queue_name: String) -> Self {
        Self { queue_name }
    }
}

#[async_trait]
impl AsyncConsumer for ConnectConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let routing_key = String::from_utf8(content).unwrap();

        let exchange_name = format!("{}-{}", MESSAGES_EXCHANGE, routing_key);
        if let Err(e) = declare_messages_exchange(channel, &routing_key)
            .await
            .map_err(|e| {
                error!("Failed to declare exchange: {:?}", e);
            })
        {
            let _ = channel
                .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), false))
                .await
                .map_err(|e| {
                    error!("Failed to reject message: {:?}", e);
                });
            return;
        }
        if let Err(e) = channel
            .queue_bind(QueueBindArguments::new(&self.queue_name, &exchange_name, "").finish())
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
            })
        {
            let _ = channel
                .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), false))
                .await
                .map_err(|e| {
                    error!("Failed to reject message: {:?}", e);
                });
        }

        if let Err(e) = channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
        {
            error!("Failed to acknowledge message: {:?}", e);
        }
    }
}
