use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{GetRelatedUsersRequest, SendInviteRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(SendInviteRequest {
        user_id: "auth0|657821f5a1e9bf99450fff22".to_string(),
        chat_id: 2,
    });
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMzAwNjA0MCwiZXhwIjoxNzAzMDkyNDQwLCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.qVGeOxdhK10BNxiDtbJypn0fw-yXoGMvazvMEV5FJwAh3HoVve9bNqBVL5meKE8TjLgPR7TujtCRV8JdBgXCr4ZXA5e8ZLQL91SkgePkMKNqhFpyd5jxgIBbW9nwa6a6Kr8iANbGcYZy5n6vjr_Qgct6zJz0sHyQPAexRaCfss9P_NoiHhkkRncdsV9d9kEL2BuuuZN8-hPy5vjDhx93V1NxQi4rUcbVVovtXTk7UKHd7VVidm2dqWtfW2CggmQm4wb8eto1ehObf3H78_TMvIwmGpvtC--uvyCUOcLhFE6pmtn6UFtV2MiSyZr7GMyWBb7REC9tnUr8omnvYYmwAA";

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.send_invite(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
