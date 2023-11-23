use crate::messenger::messenger_server::MessengerServer;
use crate::server::crab_messenger::{
    build_crab_messenger_module, CrabMessenger, CrabMessengerImpl, CrabMessengerModule,
    MessengerAdapter, ResponseStream,
};
use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use shaku::{module, Component, Interface};
use std::env;
use std::sync::Arc;
use tonic::transport::Server as TonicServer;

mod crab_messenger;
mod persistence;

#[async_trait]
pub trait Server: Interface {
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()>;
}

#[derive(Component)]
#[shaku(interface = Server)]
pub struct ServerImpl {
    #[shaku(inject)]
    crab_messenger: Arc<dyn CrabMessenger<ChatStream = ResponseStream>>,
}

#[async_trait]
impl Server for ServerImpl {
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()> {
        let addr = "[::1]:50051".parse().unwrap();

        let messenger_adapter = MessengerAdapter::new(self.crab_messenger.clone());

        let messenger = MessengerServer::new(messenger_adapter);

        TonicServer::builder()
            .add_service(messenger)
            .serve(addr)
            .await?;

        Ok(())
    }
}

module! {
    pub ServerModule {
        components = [ServerImpl],
        providers = [],
        use CrabMessengerModule {
            components = [dyn CrabMessenger<ChatStream = ResponseStream>],
            providers = [],
        }
    }
}

pub fn build_server_module() -> Arc<ServerModule> {
    Arc::new(ServerModule::builder(build_crab_messenger_module()).build())
}
