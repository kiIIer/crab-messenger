use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::messenger::{
    Chat as GChat, Chats, CreateChatRequest, CreateChatResponse, GetUserChatsRequest,
};
use crate::utils::persistence::chat::{Chat, InsertChat};
use crate::utils::persistence::schema::{chats, users_chats};
use crate::utils::persistence::users_chats::UsersChats;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::CHAT_CONNECT_EXCHANGE;
use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::QueryDsl;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, error, instrument};

#[async_trait]
pub trait ChatManager: Interface {
    async fn get_user_chats(
        &self,
        request: Request<GetUserChatsRequest>,
    ) -> Result<Response<Chats>, Status>;

    async fn create_chat(
        &self,
        request: Request<CreateChatRequest>,
    ) -> Result<Response<CreateChatResponse>, Status>;
}

#[derive(Component)]
#[shaku(interface = ChatManager)]
pub struct ChatManagerImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,

    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[async_trait]
impl ChatManager for ChatManagerImpl {
    #[tracing::instrument(skip(self, request), err)]
    async fn get_user_chats(
        &self,
        request: Request<GetUserChatsRequest>,
    ) -> Result<Response<Chats>, Status> {
        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection: {}", e);
            Status::internal("Failed to get DB connection")
        })?;

        let metadata = request.metadata();
        let user_id = metadata.get("user_id").unwrap().to_str().unwrap();
        debug!("User_id: {:?}", user_id);

        let chats = users_chats::table
            .filter(users_chats::user_id.eq(user_id))
            .inner_join(chats::table.on(users_chats::chat_id.eq(chats::id)))
            .select(chats::all_columns)
            .load::<Chat>(&mut connection)
            .map_err(|e| {
                error!("Failed to get chats: {}", e);
                Status::internal("Failed to get chats")
            })?;

        let chats: Vec<GChat> = chats.into_iter().map(|c| c.into()).collect();

        Ok(Response::new(Chats { chats }))
    }

    #[instrument(skip(self, request), err)]
    async fn create_chat(
        &self,
        request: Request<CreateChatRequest>,
    ) -> Result<Response<CreateChatResponse>, Status> {
        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection: {}", e);
            Status::internal("Failed to get DB connection")
        })?;

        let metadata = request.metadata();
        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        debug!("User_id: {:?}", user_id);
        let chat_name = request.into_inner().name;

        let insert_chat = InsertChat { name: chat_name };
        let chat = diesel::insert_into(chats::table)
            .values(insert_chat)
            .get_result::<Chat>(&mut connection)
            .map_err(|e| {
                error!("Failed to create chat: {}", e);
                Status::internal("Failed to create chat")
            })?;

        let user_chat = UsersChats {
            user_id: user_id.to_string(),
            chat_id: chat.id,
        };

        diesel::insert_into(users_chats::table)
            .values(user_chat)
            .execute(&mut connection)
            .map_err(|e| {
                error!("Failed to insert user_chat: {}", e);
                Status::internal("Failed to insert user_chat")
            })?;

        let channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {}", e);
            Status::internal("Failed to get channel")
        })?;

        channel
            .basic_publish(
                BasicProperties::default(),
                chat.id.to_string().into_bytes(),
                BasicPublishArguments::new(CHAT_CONNECT_EXCHANGE, &user_id)
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish message: {}", e);
                Status::internal("Failed to publish message")
            })?;

        Ok(Response::new(CreateChatResponse {
            chat: Some(chat.into()),
        }))
    }
}

module! {
    pub ChatManagerModule {
        components = [ChatManagerImpl],
        providers = [],
        use DBConnectionManagerModule{
            components = [dyn DBConnectionManager],
            providers = [],
        },
        use ChannelManagerModule{
            components = [dyn ChannelManager],
            providers = [],
        },
    }
}
pub fn build_chat_manager_module() -> Arc<ChatManagerModule> {
    Arc::new(
        ChatManagerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
        )
        .build(),
    )
}
