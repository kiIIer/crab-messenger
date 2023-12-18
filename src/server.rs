use std::sync::Arc;

use async_trait::async_trait;
use shaku::{module, Component, Interface};
use tonic::transport::Server as TonicServer;
use tonic_async_interceptor::async_interceptor;
use tracing::info;

use crate::server::auth_interceptor::{
    build_auth_interceptor_module, AuthInterceptorFactory, AuthInterceptorModule,
};
use crate::server::crab_messenger::{
    build_crab_messenger_module, CrabMessenger, CrabMessengerModule, MessengerAdapter,
    ChatResponseStream,
};
use crate::utils::messenger::messenger_server::MessengerServer;

mod auth_interceptor;
mod crab_messenger;

#[async_trait]
pub trait Server: Interface {
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()>;
}

#[derive(Component)]
#[shaku(interface = Server)]
pub struct ServerImpl {
    #[shaku(inject)]
    crab_messenger: Arc<dyn CrabMessenger<ChatStream =ChatResponseStream>>,

    #[shaku(inject)]
    auth_interceptor_factory: Arc<dyn AuthInterceptorFactory>,
}

#[async_trait]
impl Server for ServerImpl {
    #[tracing::instrument(skip(self), err)]
    async fn run_server(self: Arc<Self>) -> anyhow::Result<()> {
        info!("Starting server");

        let addr = "[::1]:50051".parse().unwrap();

        let messenger_adapter = MessengerAdapter::new(self.crab_messenger.clone());
        let auth_interceptor = self.auth_interceptor_factory.create();
        let interceptor_layer = async_interceptor(move |req| {
            let interceptor = auth_interceptor.clone();
            async move { interceptor.intercept(req).await }
        });

        let messenger = MessengerServer::new(messenger_adapter);

        TonicServer::builder()
            .layer(interceptor_layer)
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
            components = [dyn CrabMessenger<ChatStream = ChatResponseStream>],
            providers = [],
        },
        use AuthInterceptorModule {
            components = [dyn AuthInterceptorFactory],
            providers = [],
        },
    }
}

pub fn build_server_module() -> Arc<ServerModule> {
    Arc::new(
        ServerModule::builder(
            build_crab_messenger_module(),
            build_auth_interceptor_module(),
        )
        .build(),
    )
}
