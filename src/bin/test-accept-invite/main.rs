use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{AnswerInviteRequest, GetInvitesRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(AnswerInviteRequest {
        invite_id: 13,
        accept: true,
    });
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMzEyOTU2NSwiZXhwIjoxNzAzMjE1OTY1LCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.qrNHbSv-LqFBrhQEFCsW__cRfLYHOMZzepNh2ARtupYwsb4_RyRSAHK2mHBt6vbmKSTPP-_U4iWh6OspPIraEywXk98rJCuDyGVXSiEgQiJV2GIkL_agjTZNXvd0ygAj--7loQN5x1pamvLtkQfwgIHZlCbP4DB7gWYxxs9SO3bHFr2CU0kHZ46YwfgT42-yVGb5AAl8PCa8QxfqLnsyxWWv3quZSAwdCnskB-EvCj_oTMAIXSM53FAmO510fE6qSI-KjnIEq_9tp3e-M_l_v57vmb9bCYgdBtKqtIGJMFpEhI-KmuTCBkTG6IaKOw67fXur2fCvr55YKkdMM_yPNg";

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
