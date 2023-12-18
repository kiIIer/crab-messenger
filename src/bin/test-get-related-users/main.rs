use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{GetRelatedUsersRequest, GetUserChatsRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(GetRelatedUsersRequest {});
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMjkxNjUzMywiZXhwIjoxNzAzMDAyOTMzLCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.dYSxPGkn4UQwRUNVlxf7IXky7Mix_Iff22_jLjFbb-L845ON7lPfp3HOmlwti6T9BFlQWhMI4YDHDTsNKSUihi22Ldbu2yEOKhTnVgBWlo2NYlXlZM0HhK2jMxJ_GuYPqX14O32F_HxEjwVwx-nMf9A-9lXyDZMQjd_nh978_dzTe-7Fz19W7XFP6oUyHmF9sJ19YHKHlK8kW1E6_VGMe6fy4vmlSTSkkhiLS_uGykE5qTfR2ZrljALeAerXyTqCuC7IQZdgAn15izyxllT3QA4CPriKUVTTdb_n75xRmFi1KEGi-Cv8EnQ0NqZwDUCy_80UE4AsD7aEiQtuc3SC2w";

    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.get_related_users(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
