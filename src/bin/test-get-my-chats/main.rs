use std::str::FromStr;
use tonic::metadata::MetadataValue;
use crab_messenger::utils::messenger::GetMyChats;
use crab_messenger::utils::messenger::messenger_client::MessengerClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(GetMyChats {});
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDI0ODY5MjMsImV4cCI6MTcwMjU3MzMyMywiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.TJBDiTmPOWVtNqXW9JJiryWS6f2rD4WTBBw5qR_pIkli-NcIwnG--gx-1FeHqwb_nNLn9hUKhLenAEIlojsmDzlrPDAcr2VQATBdyOTDugY-e3L1lYGgoEImW91un0URqCaNArQ3hj_68OKK2EzafQzEaJHvp8TECXMQnhIFpZTZUiqM2rLcvZjqd3xi9Sy8b0ICC5vh7jzN7Lx-nXZ5KxihvN3TYHeLmaeSow_oKxGoKjtWdjjBl-VPvrnWb_CxWMVjjGpbU5uMLtJHHIK-lcvWmCwLtPrSiT2U88v0JNrHJIZRj6f5CgmS8OFBYi5N3gTJzsnKh_EUsVvJFw34LQ";

    // Create a MetadataValue from the token
    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| anyhow::Error::new(e))?;
    request
        .metadata_mut()
        .insert("authorization", token_metadata);

    let response = client.get_user_chats(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}