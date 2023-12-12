use std::sync::Arc;

use amqprs::channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments};
use async_trait::async_trait;
use shaku::{module, Component, Interface};
use tokio::signal;
use tracing::{error, info, Instrument};

use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{
    declare_messages_exchange, declare_new_message_exchange, setup_error_handling,
    NEW_MESSAGE_EXCHANGE,
};
use crate::worker::new_message_consumer::NewMessageConsumer;

mod new_message_consumer;

#[async_trait]
pub trait Worker: Interface {
    async fn run_worker(self: Arc<Self>) -> anyhow::Result<()>;
}

#[derive(Component)]
#[shaku(interface = Worker)]
pub struct WorkerImpl {
    #[shaku(inject)]
    connection_manager: Arc<dyn DBConnectionManager>,

    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[async_trait]
impl Worker for WorkerImpl {
    #[tracing::instrument(skip(self), err)]
    async fn run_worker(self: Arc<Self>) -> anyhow::Result<()> {
        info!("Starting worker");

        let channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            e
        })?;

        declare_new_message_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            e
        })?;

        declare_messages_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            e
        })?;

        setup_error_handling(&channel).await.map_err(|e| {
            error!("Failed to setup error handling: {:?}", e);
            e
        })?;

        let queue_name = "new_message_queue";
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(queue_name))
            .await
            .map_err(|e| {
                error!("Failed to declare queue: {:?}", e);
                e
            })?;

        channel
            .queue_bind(QueueBindArguments::new(
                queue_name,
                NEW_MESSAGE_EXCHANGE,
                "",
            ))
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                e
            })?;

        let consumer = NewMessageConsumer::new(self.connection_manager.clone());
        let args = BasicConsumeArguments::new(queue_name, "worker");

        channel.basic_consume(consumer, args).await.map_err(|e| {
            error!("Failed to consume: {:?}", e);
            e
        })?;

        signal::ctrl_c().await.map_err(|e| {
            error!("Failed to wait for ctrl-c: {:?}", e);
            e
        })?;

        Ok(())
    }
}

module! {
    pub WorkerModule{
        components = [WorkerImpl],
        providers = [],
        use DBConnectionManagerModule {
            components = [dyn DBConnectionManager],
            providers = [],
        },
        use ChannelManagerModule {
            components = [dyn ChannelManager],
            providers = [],
        },
    }
}

pub fn build_worker_module() -> Arc<WorkerModule> {
    Arc::new(
        WorkerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
        )
        .build(),
    )
}
