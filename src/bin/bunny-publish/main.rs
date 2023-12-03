use amqprs::{
    channel::{BasicPublishArguments, ExchangeDeclareArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use tokio::{
    self,
    io::{self, AsyncBufReadExt},
};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // Open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "my_user",
        "my_password", // Replace with your actual password
    ))
    .await
    .unwrap();

    // Open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();

    // Declare the exchange
    channel
        .exchange_declare(ExchangeDeclareArguments::new("Messages", "direct"))
        .await
        .unwrap();

    channel
        .exchange_declare(ExchangeDeclareArguments::new("ConnectCommand", "direct"))
        .await
        .unwrap();
    let chat_id = "anime";
    let args = BasicPublishArguments::new("ConnectCommand", "");
    channel
        .basic_publish(
            BasicProperties::default(),
            chat_id.to_string().into_bytes(),
            args,
        )
        .await
        .unwrap();

    // Create an asynchronous reader for stdin
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();

    // Read lines from stdin and publish them
    while let Ok(Some(line)) = reader.next_line().await {
        // Prepare the content as a byte array
        let content = line.into_bytes();

        // Create arguments for basic_publish
        let args = BasicPublishArguments::new("Messages", chat_id); // Routing key can be empty for direct exchange

        // Publish the message
        channel
            .basic_publish(BasicProperties::default(), content, args)
            .await
            .unwrap();

        println!("Message published to exchange 'Messages'");
    }

    // Close the channel and connection
    channel.close().await.unwrap();
    connection.close().await.unwrap();
}
