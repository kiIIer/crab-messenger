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
    // Mike
    let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwODcwNjE4MTUyMTYyMjc4MzgzMyIsImF1ZCI6WyJjcmFiLWFwaSIsImh0dHBzOi8vY3JhYi1tZXNzZW5nZXIuZXUuYXV0aDAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTcwMzEyOTU2NSwiZXhwIjoxNzAzMjE1OTY1LCJhenAiOiJhODVYYzVKeXFOM2c1N1dQVUxWdTRqVE92amhOV2JXbSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MifQ.qrNHbSv-LqFBrhQEFCsW__cRfLYHOMZzepNh2ARtupYwsb4_RyRSAHK2mHBt6vbmKSTPP-_U4iWh6OspPIraEywXk98rJCuDyGVXSiEgQiJV2GIkL_agjTZNXvd0ygAj--7loQN5x1pamvLtkQfwgIHZlCbP4DB7gWYxxs9SO3bHFr2CU0kHZ46YwfgT42-yVGb5AAl8PCa8QxfqLnsyxWWv3quZSAwdCnskB-EvCj_oTMAIXSM53FAmO510fE6qSI-KjnIEq_9tp3e-M_l_v57vmb9bCYgdBtKqtIGJMFpEhI-KmuTCBkTG6IaKOw67fXur2fCvr55YKkdMM_yPNg";

    // Shata
    // let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDMxMzAyNDYsImV4cCI6MTcwMzIxNjY0NiwiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.CE9hi9s5bgPYK_V2qWhgAhDs77jWArnbBLr7Amjj2nDkJA3K8e1KcgMjtvUhyg0uCG-dwGm6xs7AZ43txqSGULeOIa31eimuh7Du3rr8OQeHdXP4ZF9XPPo_cJ8rf6WyV2UZGmnFe9Eekg7imV2QonoBZ_BLvvzxepWqP3NSWiCPVidNwKtU8hozLRgwM9Gbs9nr6ecUm7qSisO7QDLD5S4plzW-CYPnuTcO0Mtc5k6YyvvHQN7-j3Vjyqecg-Px5YHoF_K-NTp-YzYWJA1z5fpmOyUgRMKqrqYSGxcP7-MVBh-Ok3Q0HCCRk1REOnhe4LiYrksQwSO_K-W1Lmd_Ww";

    // let auth_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjdhXzJnWkdRSmoxOFZWRmV2Z3VMeSJ9.eyJpc3MiOiJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS8iLCJzdWIiOiJhdXRoMHw2NTc4MjFmNWExZTliZjk5NDUwZmZmMjIiLCJhdWQiOlsiY3JhYi1hcGkiLCJodHRwczovL2NyYWItbWVzc2VuZ2VyLmV1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3MDMwNzgwNTQsImV4cCI6MTcwMzE2NDQ1NCwiYXpwIjoiYTg1WGM1SnlxTjNnNTdXUFVMVnU0alRPdmpoTldiV20iLCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIn0.WBxA22FcB-gYsyLsRycdXer7ufkP0hYtQd6aDqoiUF9dKgXtoIZ6AP3pSwxCrDltsiektXX_oow1mSpAgtvUxJ-RoRQMJu6r-8QgYGTEJhUMu_Ags-NLoy56RgX0MwMrkTOdG2VvKHogREJz3w1LOJcV7hZNC1vyk2rc1Q-K9qA2UqosijHmkTarbDWYSTnHlB9kv-N4rqbNexaunE0wtTe6uwpLb1Ghum9P_mKMsTRQUJb7TaWEAr2woQP5tpiFa-81CRTnn9eB6K64tZSmKF6fK3MIB-8ylkl1SvWz_4InkuijSfcw37-ryZXJO5nzT6sL5tO0HAiK8EMX7pD9Gg";

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
            chat_id: 5,
        };

        tx.send(send_msg).await.expect("Failed to send message");
        input.clear();
    }

    Ok(())
}
