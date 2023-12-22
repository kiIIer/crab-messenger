use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{GetRelatedUsersRequest, SendInviteRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(SendInviteRequest {
        user_id: "google-oauth2|108706181521622783833".to_string(),
        chat_id: 4,
    });
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDMxMzAyNDYsImV4cCI6MTcwMzIxNjY0NiwiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.CE9hi9s5bgPYK_V2qWhgAhDs77jWArnbBLr7Amjj2nDkJA3K8e1KcgMjtvUhyg0uCG-dwGm6xs7AZ43txqSGULeOIa31eimuh7Du3rr8OQeHdXP4ZF9XPPo_cJ8rf6WyV2UZGmnFe9Eekg7imV2QonoBZ_BLvvzxepWqP3NSWiCPVidNwKtU8hozLRgwM9Gbs9nr6ecUm7qSisO7QDLD5S4plzW-CYPnuTcO0Mtc5k6YyvvHQN7-j3Vjyqecg-Px5YHoF_K-NTp-YzYWJA1z5fpmOyUgRMKqrqYSGxcP7-MVBh-Ok3Q0HCCRk1REOnhe4LiYrksQwSO_K-W1Lmd_Ww";

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.send_invite(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
