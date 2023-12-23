use amqprs::channel::Channel;
use amqprs::connection::{Connection, OpenConnectionArguments};
use anyhow::{Error, Result};
use async_trait::async_trait;
use dotenv::dotenv;
use shaku::{module, Component, Interface};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait ChannelManager: Interface {
    async fn get_channel(&self) -> Result<Channel, anyhow::Error>;
}

#[derive(Component)]
#[shaku(interface = ChannelManager)]
pub struct ChannelManagerImpl {
    connection: Arc<Mutex<Option<Connection>>>,
    connection_args: OpenConnectionArguments,
}

impl ChannelManagerImpl {
    async fn establish_connection(
        &self,
        connection_lock: &mut Option<Connection>,
    ) -> Result<(), anyhow::Error> {
        let new_connection = Connection::open(&self.connection_args)
            .await
            .map_err(anyhow::Error::new)?;
        *connection_lock = Some(new_connection);
        Ok(())
    }
}

#[async_trait]
impl ChannelManager for ChannelManagerImpl {
    async fn get_channel(&self) -> Result<Channel, anyhow::Error> {
        // Lock the connection just once
        let mut connection = self.connection.lock().await;

        if connection.is_none() {
            self.establish_connection(&mut connection).await?;
        }

        match connection.as_mut().unwrap().open_channel(None).await {
            Ok(channel) => Ok(channel),
            Err(_) => {
                // If opening a channel fails, re-establish the connection and retry
                self.establish_connection(&mut connection).await?;
                connection
                    .as_mut()
                    .unwrap()
                    .open_channel(None)
                    .await
                    .map_err(anyhow::Error::new)
            }
        }
    }
}

module! {
    pub ChannelManagerModule{
        components = [ChannelManagerImpl],
        providers = [],
    }
}

pub fn build_channel_manager_module() -> Arc<ChannelManagerModule> {
    dotenv().ok();

    let rabbit_host = env::var("RABBIT_HOST").expect("RABBIT_HOST must be set");
    let rabbit_port = env::var("RABBIT_PORT")
        .expect("RABBIT_PORT must be set")
        .parse()
        .expect("RABBIT_PORT must be a number");
    let rabbit_user = env::var("RABBIT_USER").expect("RABBIT_USER must be set");
    let rabbit_password = env::var("RABBIT_PASSWORD").expect("RABBIT_PASSWORD must be set");

    let connection_args =
        OpenConnectionArguments::new(&rabbit_host, rabbit_port, &rabbit_user, &rabbit_password);

    let connection = Arc::new(Mutex::new(None));
    Arc::new(
        ChannelManagerModule::builder()
            .with_component_parameters::<ChannelManagerImpl>(ChannelManagerImplParameters {
                connection,
                connection_args,
            })
            .build(),
    )
}
