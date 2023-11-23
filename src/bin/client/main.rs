use crab_messenger::messenger::{messenger_client::MessengerClient, GetMessages, Message};
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH)?;
    let timestamp = Timestamp {
        seconds: since_the_epoch.as_secs() as i64,
        nanos: since_the_epoch.subsec_nanos() as i32,
    };

    let request = tonic::Request::new(GetMessages {
        chat_id: 1,
        created_before: Some(timestamp),
    });

    let response = client.get_messages(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
