use amqprs::channel::{Channel, ExchangeDeclareArguments, ExchangeType};
use amqprs::error::Error;
use amqprs::FieldTable;

pub const NEW_MESSAGE_EXCHANGE: &str = "NewMessageExchange";
pub const MESSAGES_EXCHANGE: &str = "MessageSExchange";

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
