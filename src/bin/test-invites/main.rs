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
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDI5NjEyMTksImV4cCI6MTcwMzA0NzYxOSwiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.kL8YdJhrqqKhuePkabW4elU0eCh2s_SAwlpG059UiHBmMyai5qkFlLWIyWNUeU5Wy-EZeNvVbzgvEjo8s5JBXTlRSO5k1VIgBznwOhwcjpD_ilnFdSy0lk2VEMJa_mRCDf9bZBDWtv7tEWlo46CqhivzUHczVWodp2pgNevT3rQLsiyNf_a5-f1qVNSCpV90innR2ShD4lJSA1Zku2LuCf3AHtRVKaeFLyku_bdzbtiFVd3L2K_-OmKmRj6Vjze86OtFpoZJ-q4UFZyVL_Zpdk28zc6wPvKXzoAU5R7O_Nv9y4SkdgOl6juugFQs6Wl7nHUNwqyCT2AlOBO26DFLrw";

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
