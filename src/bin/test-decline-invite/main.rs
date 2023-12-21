use crab_messenger::utils::messenger::messenger_client::MessengerClient;
use crab_messenger::utils::messenger::{AnswerInviteRequest, GetInvitesRequest};
use std::str::FromStr;
use tonic::metadata::MetadataValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(AnswerInviteRequest {
        invite_id: 1,
        accept: false,
    });
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDMwNzgwNTQsImV4cCI6MTcwMzE2NDQ1NCwiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.WBxA22FcB-gYsyLsRycdXer7ufkP0hYtQd6aDqoiUF9dKgXtoIZ6AP3pSwxCrDltsiektXX_oow1mSpAgtvUxJ-RoRQMJu6r-8QgYGTEJhUMu_Ags-NLoy56RgX0MwMrkTOdG2VvKHogREJz3w1LOJcV7hZNC1vyk2rc1Q-K9qA2UqosijHmkTarbDWYSTnHlB9kv-N4rqbNexaunE0wtTe6uwpLb1Ghum9P_mKMsTRQUJb7TaWEAr2woQP5tpiFa-81CRTnn9eB6K64tZSmKF6fK3MIB-8ylkl1SvWz_4InkuijSfcw37-ryZXJO5nzT6sL5tO0HAiK8EMX7pD9Gg";

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
