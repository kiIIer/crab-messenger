use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_core::Stream;
use shaku::{module, Component, Interface};
use tonic::{Request, Response, Status, Streaming};

use crate::server::crab_messenger::message_manager::{
    build_message_manager_module, MessageManager, MessageManagerModule,
};
use crate::server::crab_messenger::user_manager::{
    build_user_manager_module, UserManager, UserManagerModule,
};
use crate::utils::messenger::messenger_server::Messenger;
use crate::utils::messenger::{
    Chat, GetMessages, GetMyChats, Message as MMessage, Messages, SendMessage,
};
use crate::utils::messenger::{GetUser, User};

mod message_manager;
mod user_manager;

pub trait CrabMessenger: Interface + Messenger {}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<MMessage, Status>> + Send>>;

#[derive(Component)]
#[shaku(interface = CrabMessenger<chatStream = ResponseStream>)]
pub struct CrabMessengerImpl {
    #[shaku(inject)]
    message_manager: Arc<dyn MessageManager<chatStream = ResponseStream>>,

    #[shaku(inject)]
    user_manager: Arc<dyn UserManager>,
}

impl CrabMessenger for CrabMessengerImpl {}

#[async_trait]
impl Messenger for CrabMessengerImpl {
    type chatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::chatStream>, Status> {
        self.message_manager.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        self.message_manager.get_messages(request).await
    }

    async fn get_user(&self, request: Request<GetUser>) -> Result<Response<User>, Status> {
        self.user_manager.get_user(request).await
    }

    async fn get_user_chats(&self, request: Request<GetMyChats>) -> Result<Response<Chat>, Status> {
        todo!()
    }
}

pub struct MessengerAdapter {
    messenger: Arc<dyn CrabMessenger<chatStream = ResponseStream>>,
}

impl MessengerAdapter {
    pub fn new(messenger: Arc<dyn CrabMessenger<chatStream = ResponseStream>>) -> Self {
        Self { messenger }
    }
}

#[async_trait]
impl Messenger for MessengerAdapter {
    type chatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::chatStream>, Status> {
        self.messenger.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        self.messenger.get_messages(request).await
    }

    async fn get_user(&self, request: Request<GetUser>) -> Result<Response<User>, Status> {
        self.messenger.get_user(request).await
    }

    async fn get_user_chats(&self, request: Request<GetMyChats>) -> Result<Response<Chat>, Status> {
        todo!()
    }
}

module! {
    pub CrabMessengerModule{
        components = [CrabMessengerImpl],
        providers = [],
        use MessageManagerModule {
            components = [dyn MessageManager<chatStream = ResponseStream>],
            providers = [],
        },
        use UserManagerModule {
            components = [dyn UserManager],
            providers = [],
        },
    }
}

pub fn build_crab_messenger_module() -> Arc<CrabMessengerModule> {
    Arc::new(
        CrabMessengerModule::builder(build_message_manager_module(), build_user_manager_module())
            .build(),
    )
}
