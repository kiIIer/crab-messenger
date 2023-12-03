use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::worker::new_message_consumer::NewMessageConsumer;
use amqprs::channel::{
    BasicConsumeArguments, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
};
use amqprs::consumer::AsyncConsumer;
use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tokio::{signal, task};

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
    async fn run_worker(self: Arc<Self>) -> anyhow::Result<()> {
        let channel = self.channel_manager.get_channel().await?;

        // Declare the exchange
        channel
            .exchange_declare(ExchangeDeclareArguments::new(
                "NewMessageExchange",
                "direct",
            ))
            .await?;

        // Declare a durable queue
        let queue_name = "new_message_queue";
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(queue_name))
            .await?;

        // Bind the queue to the exchange
        channel
            .queue_bind(QueueBindArguments::new(
                queue_name,
                "NewMessageExchange",
                "",
            ))
            .await?;

        let consumer = NewMessageConsumer::new(self.connection_manager.clone());
        let args = BasicConsumeArguments::new(queue_name, "worker");

        channel.basic_consume(consumer, args).await?;

        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

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
