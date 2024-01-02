use anyhow::Error;
use anyhow::Result;
use dotenv::dotenv;
use futures::stream::StreamExt;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::{Request, Status};

use crab_messenger::utils::messenger::{messenger_client::MessengerClient, SendMessage};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let (mut tx, rx) = mpsc::channel(4);
    dotenv().ok();

    // let auth_token = std::env::var("TEST_TOKEN_1").expect("Failed to get token");
    let auth_token = std::env::var("TEST_TOKEN_2").expect("Failed to get token");
    let chat_id = 1;

    // Create a MetadataValue from the token
    let token_metadata =
        MetadataValue::from_str(&format!("{}", auth_token)).map_err(|e| Error::new(e))?;

    // Spawn a task to handle incoming messages
    tokio::spawn(async move {
        // Create a new Request and attach the token as metadata
        let mut request = Request::new(ReceiverStream::new(rx));
        request
            .metadata_mut()
            .insert("authorization", token_metadata);

        let response_stream = client
            .chat(request)
            .await
            .expect("Failed to start chat")
            .into_inner();

        response_stream
            .for_each(|message| async {
                match message {
                    Ok(msg) => println!("\nReceived message: {:?}\n", msg),
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            })
            .await;
    });

    // Blocking read from stdin in the main function
    let mut input = String::new();
    while let Ok(_) = std::io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed == "exit" {
            // User types "exit" to gracefully close the client
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let send_msg = SendMessage {
            text: trimmed.to_string(),
            chat_id,
        };

        tx.send(send_msg).await.expect("Failed to send message");
        input.clear();
    }

    Ok(())
}
