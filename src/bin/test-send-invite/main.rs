use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{GetRelatedUsersRequest, SendInviteRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;
    dotenv::dotenv().ok();
    let auth_token = std::env::var("TEST_TOKEN_1").expect("Failed to get token");
    let user_id = "auth0|657821f5a1e9bf99450fff22".to_string();
    let chat_id = 5;

    let mut request = tonic::Request::new(SendInviteRequest { user_id, chat_id });

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.send_invite(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
