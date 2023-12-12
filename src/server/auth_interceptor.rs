use crate::utils::db_connection_manager::DBConnectionManager;
use shaku::{Component, Interface};
use std::sync::Arc;
use tonic::{Request, Status};

pub trait AuthInterceptor: Interface {
    fn intercept(&self, request: Request<()>) -> Result<Request<()>, Status>;
}

#[derive(Component)]
#[shaku(interface = AuthInterceptor)]
pub struct AuthInterceptorImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
}

impl AuthInterceptor for AuthInterceptorImpl {
    fn intercept(&self, request: Request<()>) -> Result<Request<()>, Status> {
        Ok(request)
    }
}
