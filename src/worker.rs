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
    declare_accept_invites_exchange, declare_invites_exchange, declare_new_message_exchange,
    declare_send_invite_exchange, setup_error_handling, ACCEPT_INVITES_EXCHANGE,
    NEW_MESSAGE_EXCHANGE, SEND_INVITE_EXCHANGE,
};
use crate::worker::new_message_consumer::NewMessageConsumer;
use crate::worker::send_invite_consumer::SendInviteConsumer;

mod accept_invite_consumer;
mod new_message_consumer;
mod send_invite_consumer;

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

        setup_error_handling(&channel).await.map_err(|e| {
            error!("Failed to setup error handling: {:?}", e);
            e
        })?;

        declare_send_invite_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            e
        })?;

        let new_message_queue = "new_message_queue";
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(
                new_message_queue,
            ))
            .await
            .map_err(|e| {
                error!("Failed to declare queue: {:?}", e);
                e
            })?;

        channel
            .queue_bind(QueueBindArguments::new(
                new_message_queue,
                NEW_MESSAGE_EXCHANGE,
                "",
            ))
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                e
            })?;

        let message_consumer = NewMessageConsumer::new(self.connection_manager.clone());
        let args = BasicConsumeArguments::new(new_message_queue, "worker-messages");

        channel
            .basic_consume(message_consumer, args)
            .await
            .map_err(|e| {
                error!("Failed to consume: {:?}", e);
                e
            })?;

        let send_invite_queue = "send_invite_queue";
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(
                send_invite_queue,
            ))
            .await
            .map_err(|e| {
                error!("Failed to declare queue: {:?}", e);
                e
            })?;

        channel
            .queue_bind(QueueBindArguments::new(
                send_invite_queue,
                SEND_INVITE_EXCHANGE,
                "",
            ))
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                e
            })?;

        let invite_consumer = SendInviteConsumer::new(self.connection_manager.clone());
        let args = BasicConsumeArguments::new(send_invite_queue, "worker-invites");

        channel
            .basic_consume(invite_consumer, args)
            .await
            .map_err(|e| {
                error!("Failed to consume: {:?}", e);
                e
            })?;

        declare_accept_invites_exchange(&channel)
            .await
            .map_err(|e| {
                error!("Failed to declare exchange: {:?}", e);
                e
            })?;
        let accept_invite_queue = "accept_invite_queue";
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(
                accept_invite_queue,
            ))
            .await
            .map_err(|e| {
                error!("Failed to declare queue: {:?}", e);
                e
            })?;

        channel
            .queue_bind(QueueBindArguments::new(
                accept_invite_queue,
                ACCEPT_INVITES_EXCHANGE,
                "",
            ))
            .await
            .map_err(|e| {
                error!("Failed to bind queue: {:?}", e);
                e
            })?;

        let accept_invite_consumer =
            accept_invite_consumer::AcceptInviteConsumer::new(self.connection_manager.clone());
        let args = BasicConsumeArguments::new(accept_invite_queue, "worker-accept-invites");
        channel
            .basic_consume(accept_invite_consumer, args)
            .await
            .map_err(|e| {
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
