use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{AnswerInviteRequest, GetInvitesRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;
    dotenv::dotenv().ok();
    let auth_token = std::env::var("TEST_TOKEN_2").expect("Failed to get token");
    let invite_id = 36;

    let mut request = tonic::Request::new(AnswerInviteRequest {
        invite_id,
        accept: false,
    });

    // Create a MetadataValue from the token
    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.answer_invite(request).await?;

    println!("RESPONSE={:?}", response);
    Ok(())
}
