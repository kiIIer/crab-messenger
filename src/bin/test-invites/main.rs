use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{InvitesRequest, SendInviteRequest};
use futures::stream::StreamExt;
use std::str::FromStr;
use tokio::signal;
use tonic::metadata::MetadataValue;
use tracing::error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(InvitesRequest {});
    dotenv::dotenv().ok();
    let auth_token = std::env::var("TEST_TOKEN_2").expect("Failed to get token");

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response_stream = client
        .invites(request)
        .await
        .expect("Failed to start invites")
        .into_inner();

    response_stream
        .for_each(|message| async {
            match message {
                Ok(msg) => println!("Received message: {:?}", msg),
                Err(e) => eprintln!("Error: {:?}", e),
            }
        })
        .await;

    signal::ctrl_c().await.map_err(|e| {
        error!("Failed to wait for ctrl-c: {:?}", e);
        e
    })?;
    Ok(())
}
