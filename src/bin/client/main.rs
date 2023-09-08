use crab_messenger::messenger::{messenger_client::MessengerClient, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(Message {
        message: "Tonic".into(),
    });

    let response = client.chat(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
