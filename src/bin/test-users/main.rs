use crab_messenger::utils::messenger::messenger_client::MessengerClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(crab_messenger::utils::messenger::GetUser {
        user_id: Some("google-oauth2|108706181521622783833".to_string()),
        email: None,
    });

    let response = client.get_user(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
