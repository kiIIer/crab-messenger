use std::sync::Arc;

use async_trait::async_trait;
use shaku::{module, Component, Interface};
use tonic::transport::Server as TonicServer;
use tonic::{Request, Status};
use tracing::{info, instrument};

use crate::server::crab_messenger::{
    build_crab_messenger_module, CrabMessenger, CrabMessengerModule, MessengerAdapter,
    ResponseStream,
};
use crate::utils::messenger::messenger_server::MessengerServer;

mod crab_messenger;
mod auth_interceptor;

#[async_trait]
pub trait Server: Interface {
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()>;
}

#[derive(Component)]
#[shaku(interface = Server)]
pub struct ServerImpl {
    #[shaku(inject)]
    crab_messenger: Arc<dyn CrabMessenger<chatStream = ResponseStream>>,
}

#[async_trait]
impl Server for ServerImpl {
    #[tracing::instrument(skip(self), err)]
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()> {
        info!("Starting server");

        let addr = "[::1]:50051".parse().unwrap();

        let messenger_adapter = MessengerAdapter::new(self.crab_messenger.clone());

        let messenger = MessengerServer::with_interceptor(messenger_adapter, intercept);

        TonicServer::builder()
            .add_service(messenger)
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[instrument(skip(req))]
fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    info!("Intercepting: {:?}", req);

    Ok(req)
}

module! {
    pub ServerModule {
        components = [ServerImpl],
        providers = [],
        use CrabMessengerModule {
            components = [dyn CrabMessenger<chatStream = ResponseStream>],
            providers = [],
        }
    }
}

pub fn build_server_module() -> Arc<ServerModule> {
    Arc::new(ServerModule::builder(build_crab_messenger_module()).build())
}
