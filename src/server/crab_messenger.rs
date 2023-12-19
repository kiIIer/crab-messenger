use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use futures_core::Stream;
use shaku::{module, Component, Interface};
use tonic::{Request, Response, Status, Streaming};

use crate::server::crab_messenger::chat_manager::{
    build_chat_manager_module, ChatManager, ChatManagerModule,
};
use crate::server::crab_messenger::invite_manager::{
    build_invite_manager_module, InviteManager, InviteManagerModule,
};
use crate::server::crab_messenger::message_manager::{
    build_message_manager_module, MessageManager, MessageManagerModule,
};
use crate::server::crab_messenger::user_manager::{
    build_user_manager_module, UserManager, UserManagerModule,
};
use crate::utils::messenger::messenger_server::Messenger;
use crate::utils::messenger::{
    Chats, GetMessagesRequest, GetRelatedUsersRequest, GetUserChatsRequest, Message as MMessage,
    Messages, SendInviteRequest, SendMessage, Users,
};
use crate::utils::messenger::{SearchUserQuery, SendInviteResponse};

mod chat_manager;
mod message_manager;
pub mod user_manager;

mod invite_manager;

pub trait CrabMessenger: Interface + Messenger {}

pub type ChatResponseStream = Pin<Box<dyn Stream<Item = Result<MMessage, Status>> + Send>>;

#[derive(Component)]
#[shaku(interface = CrabMessenger<ChatStream = ChatResponseStream>)]
pub struct CrabMessengerImpl {
    #[shaku(inject)]
    message_manager: Arc<dyn MessageManager<ChatStream = ChatResponseStream>>,

    #[shaku(inject)]
    user_manager: Arc<dyn UserManager>,

    #[shaku(inject)]
    invite_manager: Arc<dyn InviteManager>,

    #[shaku(inject)]
    chat_manager: Arc<dyn ChatManager>,
}

impl CrabMessenger for CrabMessengerImpl {}

#[async_trait]
impl Messenger for CrabMessengerImpl {
    type ChatStream = ChatResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        self.message_manager.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<Messages>, Status> {
        self.message_manager.get_messages(request).await
    }

    async fn search_user(
        &self,
        request: Request<SearchUserQuery>,
    ) -> Result<Response<Users>, Status> {
        self.user_manager.search_user(request).await
    }

    async fn get_user_chats(
        &self,
        request: Request<GetUserChatsRequest>,
    ) -> Result<Response<Chats>, Status> {
        self.chat_manager.get_user_chats(request).await
    }

    async fn get_related_users(
        &self,
        request: Request<GetRelatedUsersRequest>,
    ) -> Result<Response<Users>, Status> {
        self.user_manager.get_related_users(request).await
    }

    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status> {
        self.invite_manager.send_invite(request).await
    }
}

pub struct MessengerAdapter {
    messenger: Arc<dyn CrabMessenger<ChatStream = ChatResponseStream>>,
}

impl MessengerAdapter {
    pub fn new(messenger: Arc<dyn CrabMessenger<ChatStream = ChatResponseStream>>) -> Self {
        Self { messenger }
    }
}

#[async_trait]
impl Messenger for MessengerAdapter {
    type ChatStream = ChatResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        self.messenger.chat(request).await
    }

    async fn get_messages(
        &self,
        request: Request<GetMessagesRequest>,
    ) -> Result<Response<Messages>, Status> {
        self.messenger.get_messages(request).await
    }

    async fn search_user(
        &self,
        request: Request<SearchUserQuery>,
    ) -> Result<Response<Users>, Status> {
        self.messenger.search_user(request).await
    }

    async fn get_user_chats(
        &self,
        request: Request<GetUserChatsRequest>,
    ) -> Result<Response<Chats>, Status> {
        self.messenger.get_user_chats(request).await
    }

    async fn get_related_users(
        &self,
        request: Request<GetRelatedUsersRequest>,
    ) -> Result<Response<Users>, Status> {
        self.messenger.get_related_users(request).await
    }

    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status> {
        self.messenger.send_invite(request).await
    }
}

module! {
    pub CrabMessengerModule{
        components = [CrabMessengerImpl],
        providers = [],
        use MessageManagerModule {
            components = [dyn MessageManager<ChatStream = ChatResponseStream>],
            providers = [],
        },
        use UserManagerModule {
            components = [dyn UserManager],
            providers = [],
        },
        use ChatManagerModule {
            components = [dyn ChatManager],
            providers = [],
        },
        use InviteManagerModule {
            components = [dyn InviteManager],
            providers = [],
        },
    }
}

pub fn build_crab_messenger_module() -> Arc<CrabMessengerModule> {
    Arc::new(
        CrabMessengerModule::builder(
            build_message_manager_module(),
            build_user_manager_module(),
            build_chat_manager_module(),
            build_invite_manager_module(),
        )
        .build(),
    )
}
