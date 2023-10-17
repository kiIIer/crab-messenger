use crate::auth::auth_error::AuthError;
use crate::auth::{Auth, StartFlowResponse};
use async_trait::async_trait;
use reqwest::Client;
use shaku::Component;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Component)]
#[shaku(interface = Auth)]
pub struct AuthImpl {
    #[shaku(default = "a85Xc5JyqN3g57WPULVu4jTOvjhNWbWm".to_string())]
    client_id: String,
    #[shaku(default = "crab-api".to_string())]
    audience: String,
    #[shaku(default = "crab-messenger.eu.auth0.com".to_string())]
    domain: String,
}

impl Default for AuthImpl {
    fn default() -> Self {
        AuthImpl {
            client_id: "a85Xc5JyqN3g57WPULVu4jTOvjhNWbWm".to_string(),
            audience: "crab-api".to_string(),
            domain: "crab-messenger.eu.auth0.com".to_string(),
        }
    }
}

#[async_trait]
impl Auth for AuthImpl {
    async fn start_device_flow(&self) -> Result<StartFlowResponse, AuthError> {
        let client = Client::new();
        let url = format!("https://{}/oauth/device/code", self.domain);

        let form_params = [("client_id", &self.client_id), ("audience", &self.audience)];

        let response_result = client
            .post(url)
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&form_params)
            .send()
            .await;

        println!("{:?}", response_result);
        match response_result {
            Ok(response) => {
                let response_json: serde_json::Value = response.json().await.unwrap();

                let device_code = response_json
                    .get("device_code")
                    .unwrap()
                    .to_string()
                    .strip_prefix("\"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap()
                    .to_string();
                println!("TO STRING DEVICE CODE: {:?}", device_code);

                let start_flow_response = StartFlowResponse {
                    device_code,
                    user_code: response_json.get("user_code").unwrap().to_string(),
                    verification_uri: response_json.get("verification_uri").unwrap().to_string(),
                    interval: response_json.get("interval").unwrap().as_i64().unwrap() as i32,
                };

                println!("GOT THIS: {:?}", start_flow_response);

                Ok(start_flow_response)
            }
            Err(err) => Err(AuthError::Other {
                description: err.to_string(),
            }),
        }
    }

    async fn poll_access_token(
        &self,
        device_code: &str,
        interval: i32,
    ) -> Result<String, AuthError> {
        // Define the URL and form parameters for the token request
        let token_url = format!("https://{}/oauth/token", self.domain);
        let form_params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", &device_code),
            ("client_id", &self.client_id),
        ];

        println!("USING THIS CODE: {:?}", form_params);

        let client = Client::new();

        loop {
            tokio::time::sleep(Duration::from_secs(interval as u64)).await;

            let response = client
                .post(&token_url)
                .header("content-type", "application/x-www-form-urlencoded")
                .form(&form_params)
                .send()
                .await;
            println!("{:?}\n\n", response);

            match response {
                Ok(response) => {
                    match response.status() {
                        reqwest::StatusCode::FORBIDDEN => {
                            continue;
                        }
                        reqwest::StatusCode::OK => {
                            let json_response: serde_json::Value = response.json().await.unwrap();
                            if let Some(access_token) = json_response.get("access_token") {
                                if let Some(access_token_str) = access_token.as_str() {
                                    // Authorization succeeded
                                    return Ok(access_token_str.to_string());
                                }
                            }
                        }
                        _ => {
                            // Handle other response statuses if necessary
                            return Err(AuthError::Other {
                                description: "Unknown error".to_string(),
                            });
                        }
                    }
                }
                Err(_) => {
                    // Handle request error if necessary
                    return Err(AuthError::Other {
                        description: "Request error".to_string(),
                    });
                }
            }
        }
    }
}
