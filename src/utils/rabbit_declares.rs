use amqprs::channel::{
    BasicPublishArguments, Channel, ExchangeDeclareArguments, ExchangeType, QueueBindArguments,
    QueueDeclareArguments,
};
use amqprs::error::Error;
use amqprs::{BasicProperties, FieldTable};

pub const NEW_MESSAGE_EXCHANGE: &str = "NewMessageExchange";
pub const MESSAGES_EXCHANGE: &str = "MessageSExchange";
pub const ERROR_EXCHANGE: &str = "ErrorExchange";
pub const ERROR_QUEUE: &str = "ErrorQueue";

pub async fn declare_new_message_exchange(channel: &Channel) -> Result<(), Error> {
    channel
        .exchange_declare(ExchangeDeclareArguments::new(
            NEW_MESSAGE_EXCHANGE,
            "direct",
        ))
        .await
}

pub async fn declare_messages_exchange(channel: &Channel) -> Result<(), Error> {
    channel
        .exchange_declare(
            ExchangeDeclareArguments::of_type(MESSAGES_EXCHANGE, ExchangeType::Fanout)
                .passive(false)
                .durable(false)
                .auto_delete(false)
                .internal(false)
                .no_wait(false)
                .arguments(FieldTable::default())
                .finish(),
        )
        .await
}

pub async fn setup_error_handling(channel: &Channel) -> Result<(), Error> {
    channel
        .exchange_declare(
            ExchangeDeclareArguments::of_type(ERROR_EXCHANGE, ExchangeType::Fanout)
                .passive(false)
                .durable(true) // Durable exchange survives broker restarts
                .auto_delete(false)
                .internal(false)
                .no_wait(false)
                .arguments(FieldTable::default())
                .finish(),
        )
        .await?;

    channel
        .queue_declare(
            QueueDeclareArguments::durable_client_named(ERROR_QUEUE)
                .arguments(FieldTable::default())
                .finish(),
        )
        .await?;

    channel
        .queue_bind(
            QueueBindArguments::new(ERROR_QUEUE, ERROR_EXCHANGE, "")
                .arguments(FieldTable::default())
                .finish(),
        )
        .await?;

    Ok(())
}
pub async fn send_to_error_queue(channel: &Channel, error_message: Vec<u8>) -> Result<(), Error> {
    channel
        .basic_publish(
            BasicProperties::default(),
            error_message,
            BasicPublishArguments::new(ERROR_EXCHANGE, "")
                .mandatory(false)
                .immediate(false)
                .finish(),
        )
        .await?;

    Ok(())
}
