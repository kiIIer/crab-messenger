use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::GetUserChatsRequest;
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(GetUserChatsRequest {});

    dotenv::dotenv().ok();
    let auth_token = std::env::var("TEST_TOKEN_1").expect("Failed to get token");

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.get_user_chats(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
