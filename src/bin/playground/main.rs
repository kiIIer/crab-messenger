use chrono::Utc;
use crab_messenger::client::{build_client_module, Client};
use crab_messenger::server::persistence::message::{InsertMessage, Message};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use crossterm::{
    execute,
    terminal::{size, ScrollUp, SetSize},
};
use shaku::HasComponent;
use std::io;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
// let module = build_client_module();
// let client: Arc<dyn Client> = module.resolve();
// client.run_client()?;
// let message = InsertMessage {
//     chat_id: 1,
//     text: "This one was without id and timestamp".to_string(),
//     user_id: "google-oauth2|108706181521622783833".to_string(),
// };

// let message = serde_json::to_string(&message)?;
// println!("Serialized message: \n {}", message);
// Ok(())
// }

use crab_messenger::utils::messenger::{messenger_client::MessengerClient, SendMessage};
use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://[::1]:50051").await?;

    let (mut tx, rx) = mpsc::channel(4);

    // Spawn a task to handle incoming messages
    tokio::spawn(async move {
        let response_stream = client
            .chat(Request::new(ReceiverStream::new(rx)))
            .await
            .expect("Failed to start chat")
            .into_inner();

        response_stream
            .for_each(|message| async {
                match message {
                    Ok(msg) => println!("Received message: {:?}", msg),
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            })
            .await;
    });

    // Blocking read from stdin in the main function
    let mut input = String::new();
    while let Ok(_) = std::io::stdin().read_line(&mut input) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let send_msg = SendMessage {
            text: trimmed.to_string(),
            chat_id: 1, // Using 1 as the chat ID
        };

        tx.send(send_msg).await.expect("Failed to send message");
        input.clear();
    }

    Ok(())
}
