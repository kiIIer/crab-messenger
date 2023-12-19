use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::messenger::{Chat as GChat, GetUserChatsRequest, Chats};
use crate::utils::persistence::chat::Chat;
use crate::utils::persistence::schema::{chats, users_chats};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::QueryDsl;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, error};

#[async_trait]
pub trait ChatManager: Interface {
    async fn get_user_chats(
        &self,
        request: Request<GetUserChatsRequest>,
    ) -> Result<Response<Chats>, Status>;
}

#[derive(Component)]
#[shaku(interface = ChatManager)]
pub struct ChatManagerImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
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
}

module! {
    pub ChatManagerModule {
        components = [ChatManagerImpl],
        providers = [],
        use DBConnectionManagerModule{
            components = [dyn DBConnectionManager],
            providers = [],
        },
    }
}

pub fn build_chat_manager_module() -> Arc<ChatManagerModule> {
    Arc::new(ChatManagerModule::builder(build_db_connection_manager_module()).build())
}
