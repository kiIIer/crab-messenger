use crate::utils::auth::auth_error::AuthError;
use crate::utils::auth::auth_impl::AuthImpl;
use async_trait::async_trait;
use serde::Deserialize;
use shaku::{module, HasComponent, Interface};
use std::sync::Arc;

pub mod auth_error;
pub mod auth_impl;

#[async_trait]
pub trait Auth: Interface {
    async fn start_device_flow(&self) -> Result<StartFlowResponse, AuthError>;
    async fn poll_access_token(
        &self,
        device_code: &str,
        interval: i32,
    ) -> Result<AuthState, AuthError>;

    async fn request_refresh_token(&self, refresh_token: &str) -> Result<AuthState, AuthError>;
}

#[derive(Deserialize, Clone)]
pub struct AuthState {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StartFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: i32,
}

module! {
    pub AuthModule {
        components = [AuthImpl],
        providers = []
    }
}

pub fn build_auth_module() -> Arc<AuthModule> {
    Arc::new(AuthModule::builder().build())
}
