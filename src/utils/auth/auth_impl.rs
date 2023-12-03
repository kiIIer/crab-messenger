use crate::utils::auth::auth_error::AuthError;
use crate::utils::auth::{Auth, AuthState, StartFlowResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use shaku::Component;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

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

        let form_params = [
            ("client_id", &self.client_id),
            ("audience", &self.audience),
            ("scope", &"openid offline_access".to_string()),
        ];

        let response = client
            .post(url)
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&form_params)
            .send()
            .await
            .map_err(|err| AuthError::Other {
                description: err.to_string(),
            })?;

        response
            .json::<StartFlowResponse>()
            .await
            .map_err(|_| AuthError::Other {
                description: "Failed to parse JSON response".to_string(),
            })
    }

    async fn poll_access_token(
        &self,
        device_code: &str,
        interval: i32,
    ) -> Result<AuthState, AuthError> {
        // Define the URL and form parameters for the token request
        let token_url = format!("https://{}/oauth/token", self.domain);
        let form_params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
            ("client_id", &self.client_id),
        ];

        let client = Client::new();

        loop {
            tokio::time::sleep(Duration::from_secs(interval as u64)).await;

            let response = client
                .post(&token_url)
                .header("content-type", "application/x-www-form-urlencoded")
                .form(&form_params)
                .send()
                .await;

            match response {
                Ok(response) => match response.status() {
                    reqwest::StatusCode::FORBIDDEN => {
                        let error_response: Result<serde_json::Value, _> = response.json().await;
                        match error_response {
                            Ok(json) => match json.get("error").and_then(|e| e.as_str()) {
                                Some("authorization_pending") => continue,
                                Some("access_denied") => return Err(AuthError::AccessDenied),
                                Some("expired_token") => return Err(AuthError::ExpiredToken),
                                _ => {
                                    return Err(AuthError::Other {
                                        description: json
                                            .get("error_description")
                                            .and_then(|e| e.as_str())
                                            .unwrap_or("Unknown error")
                                            .to_string(),
                                    })
                                }
                            },
                            Err(_) => {
                                return Err(AuthError::Other {
                                    description: "Failed to parse error response".to_string(),
                                });
                            }
                        }
                    }
                    reqwest::StatusCode::OK => {
                        let token_response: Result<AuthState, _> = response.json().await;
                        return match token_response {
                            Ok(data) => Ok(data),
                            Err(_) => Err(AuthError::Other {
                                description: "Failed to parse token response".to_string(),
                            }),
                        };
                    }
                    _ => {
                        return Err(AuthError::Other {
                            description: "Unexpected response status".to_string(),
                        });
                    }
                },
                Err(err) => {
                    return Err(AuthError::Other {
                        description: err.to_string(),
                    });
                }
            }
        }
    }

    async fn request_refresh_token(&self, refresh_token: &str) -> Result<AuthState, AuthError> {
        let token_url = format!("https://{}/oauth/token", self.domain);

        let client = reqwest::Client::new();
        let form_params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.client_id),
            ("refresh_token", refresh_token),
        ];

        let response = client
            .post(&token_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&form_params)
            .send()
            .await
            .map_err(|err| AuthError::Other {
                description: format!("Request error: {}", err),
            })?;

        if response.status() != reqwest::StatusCode::OK {
            let error_response: ErrorResponse =
                response.json().await.map_err(|err| AuthError::Other {
                    description: format!("Failed to parse error response: {}", err),
                })?;

            match error_response.error.as_str() {
                "authorization_pending" => return Err(AuthError::AccessDenied),
                "access_denied" => return Err(AuthError::AccessDenied),
                "expired_token" => return Err(AuthError::ExpiredToken),
                _ => {
                    return Err(AuthError::Other {
                        description: format!(
                            "{}: {}",
                            error_response.error,
                            error_response
                                .error_description
                                .unwrap_or_else(|| "Unknown error".to_string())
                        ),
                    })
                }
            }
        }

        let poll_response: AuthState = response.json().await.map_err(|err| AuthError::Other {
            description: format!("Failed to parse JSON response: {}", err),
        })?;
        Ok(poll_response)
    }
}
