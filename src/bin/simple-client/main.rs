use anyhow::Error;
use anyhow::Result;
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
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMjkxNjUzMywiZXhwIjoxNzAzMDAyOTMzLCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.dYSxPGkn4UQwRUNVlxf7IXky7Mix_Iff22_jLjFbb-L845ON7lPfp3HOmlwti6T9BFlQWhMI4YDHDTsNKSUihi22Ldbu2yEOKhTnVgBWlo2NYlXlZM0HhK2jMxJ_GuYPqX14O32F_HxEjwVwx-nMf9A-9lXyDZMQjd_nh978_dzTe-7Fz19W7XFP6oUyHmF9sJ19YHKHlK8kW1E6_VGMe6fy4vmlSTSkkhiLS_uGykE5qTfR2ZrljALeAerXyTqCuC7IQZdgAn15izyxllT3QA4CPriKUVTTdb_n75xRmFi1KEGi-Cv8EnQ0NqZwDUCy_80UE4AsD7aEiQtuc3SC2w";

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
        if trimmed == "exit" {
            // User types "exit" to gracefully close the client
            break;
        }
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
