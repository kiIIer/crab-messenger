use std::pin::Pin;
use std::sync::Arc;

use crate::server::chat_manager::{build_chat_manager_module, ChatManager, ChatManagerModule};
use crate::utils::messenger::messenger_server::Messenger;
use crate::utils::messenger::{GetMessages, Message as MMessage, Messages, SendMessage};

use async_trait::async_trait;
use diesel::prelude::*;
use futures_core::Stream;
use shaku::{module, Component, Interface};
use tonic::{Request, Response, Status, Streaming};

pub trait CrabMessenger: Interface + Messenger {}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<MMessage, Status>> + Send>>;

#[derive(Component)]
#[shaku(interface = CrabMessenger<ChatStream = ResponseStream>)]
pub struct CrabMessengerImpl {
    #[shaku(inject)]
    chat_manager: Arc<dyn ChatManager<ChatStream = ResponseStream>>,
}

impl CrabMessenger for CrabMessengerImpl {}

#[async_trait]
impl Messenger for CrabMessengerImpl {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        self.chat_manager.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        self.chat_manager.get_messages(request).await
    }
}

pub struct MessengerAdapter {
    messenger: Arc<dyn CrabMessenger<ChatStream = ResponseStream>>,
}

impl MessengerAdapter {
    pub fn new(messenger: Arc<dyn CrabMessenger<ChatStream = ResponseStream>>) -> Self {
        Self { messenger }
    }
}

#[async_trait]
impl Messenger for MessengerAdapter {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        self.messenger.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        self.messenger.get_messages(request).await
    }
}

module! {
    pub CrabMessengerModule{
        components = [CrabMessengerImpl],
        providers = [],
        use ChatManagerModule {
            components = [dyn ChatManager<ChatStream = ResponseStream>],
            providers = [],
        },
    }
}

pub fn build_crab_messenger_module() -> Arc<CrabMessengerModule> {
    Arc::new(CrabMessengerModule::builder(build_chat_manager_module()).build())
}
