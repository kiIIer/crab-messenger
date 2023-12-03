use amqprs::channel::{BasicAckArguments, ExchangeDeclareArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties, Deliver,
};
use tokio::time;
use tracing::{error, info, instrument};

struct MessageConsumer;

#[async_trait::async_trait]
impl AsyncConsumer for MessageConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        let message = String::from_utf8_lossy(&content);
        info!("Message received at {}: {}", deliver.routing_key(), message);

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}

struct CommandConsumer {
    exchange_name: String,
}

#[async_trait::async_trait]
impl AsyncConsumer for CommandConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        let routing_key = String::from_utf8_lossy(&content);
        info!("Received routing key: {}", routing_key);

        // Now consume from the first exchange with this routing key
        consume_from_exchange(&channel, &self.exchange_name, &routing_key).await;

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}

async fn consume_from_exchange(channel: &Channel, exchange_name: &str, routing_key: &str) {
    let queue_name = format!("{}_queue", routing_key);
    channel
        .queue_declare(QueueDeclareArguments::durable_client_named(&queue_name))
        .await
        .unwrap();

    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await
        .unwrap();

    let args = BasicConsumeArguments::new(&queue_name, ""); // Use the queue name, not the exchange name
    let message_consumer = MessageConsumer {};
    channel.basic_consume(message_consumer, args).await.unwrap();
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    tracing_subscriber::fmt::init();

    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "my_user",
        "my_password",
    ))
    .await
    .unwrap();

    let channel = connection.open_channel(None).await.unwrap();

    let command_exchange = "ConnectCommand";
    let messages_exchange = "Messages";

    channel
        .exchange_declare(ExchangeDeclareArguments::new(command_exchange, "direct"))
        .await
        .unwrap();

    channel
        .exchange_declare(ExchangeDeclareArguments::new(messages_exchange, "direct"))
        .await
        .unwrap();

    // Setup the consumer for the ConnectCommand exchange
    let command_consumer = CommandConsumer {
        exchange_name: messages_exchange.to_string(),
    };
    channel
        .queue_declare(QueueDeclareArguments::durable_client_named(
            command_exchange,
        ))
        .await
        .unwrap();
    channel
        .queue_bind(QueueBindArguments::new(
            command_exchange,
            command_exchange,
            "",
        ))
        .await
        .unwrap();

    let args = BasicConsumeArguments::new(command_exchange, "command_consumer");
    channel.basic_consume(command_consumer, args).await.unwrap();

    // Keep the channel and connection open to receive messages
    time::sleep(time::Duration::from_secs(10000)).await;

    // Close the channel and connection
    channel.close().await.unwrap();
    connection.close().await.unwrap();
}
