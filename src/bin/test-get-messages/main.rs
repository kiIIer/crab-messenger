use std::str::FromStr;
use crab_messenger::utils::messenger::{messenger_client::MessengerClient, GetMessagesRequest, Message};
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;
    dotenv::dotenv().ok();
    let auth_token = std::env::var("TEST_TOKEN_1").expect("Failed to get token");

    let chat_id = 1;

    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH)?;
    let timestamp = Timestamp {
        seconds: since_the_epoch.as_secs() as i64,
        nanos: since_the_epoch.subsec_nanos() as i32,
    };

    let mut request = tonic::Request::new(GetMessagesRequest {
        chat_id,
        created_before: Some(timestamp),
    });


    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.get_messages(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
