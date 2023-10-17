use crate::auth::auth_error::AuthError;
use crate::auth::auth_impl::AuthImpl;
use async_trait::async_trait;
use shaku::{module, HasComponent, Interface};

pub mod auth_error;
pub mod auth_impl;

#[async_trait]
pub trait Auth: Interface {
    async fn start_device_flow(&self) -> Result<StartFlowResponse, AuthError>;
    async fn poll_access_token(
        &self,
        device_code: &str,
        interval: i32,
    ) -> Result<String, AuthError>;
}

#[derive(Debug)]
pub struct StartFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: i32,
}

pub trait AuthModule: HasComponent<dyn Auth> {}

module! {
    pub AuthModuleImpl: AuthModule {
        components = [AuthImpl],
        providers = []
    }
}
