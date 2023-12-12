use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};
use shaku::{module, Component, Interface};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::utils::db_connection_manager::{
    build_db_connection_manager_module, DBConnectionManager, DBConnectionManagerModule,
};
use crate::utils::messenger::{GetUser, User as GUser};
use crate::utils::persistence::schema::users;
use crate::utils::persistence::user::User as DBUser;
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};

#[async_trait]
pub trait UserManager: Interface {
    async fn get_user(&self, request: Request<GetUser>) -> Result<Response<GUser>, Status>;
    async fn create_user(&self, user: DBUser) -> Result<(), anyhow::Error>;
}

#[derive(Component)]
#[shaku(interface = UserManager)]
pub struct UserManagerImpl {
    #[shaku(inject)]
    db_connection_manager: Arc<dyn DBConnectionManager>,
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[async_trait]
impl UserManager for UserManagerImpl {
    #[tracing::instrument(skip(self, request), err)]
    async fn get_user(&self, request: Request<GetUser>) -> Result<Response<GUser>, Status> {
        info!("Getting user");

        let get_user_req = request.into_inner();

        let mut connection = self.db_connection_manager.get_connection().map_err(|e| {
            error!("Failed to get DB connection: {}", e);
            Status::internal("Failed to get DB connection")
        })?;

        let user_result = match (get_user_req.user_id, get_user_req.email) {
            (Some(user_id), _) => {
                debug!("Querying user by id: {}", user_id);
                users::table
                    .filter(users::id.eq(user_id))
                    .first::<DBUser>(&mut connection)
            }
            (_, Some(email)) => {
                debug!("Querying user by email: {}", email);
                users::table
                    .filter(users::email.eq(email))
                    .first::<DBUser>(&mut connection)
            }
            _ => {
                error!("No user identifier provided");
                return Err(Status::invalid_argument("No user identifier provided"));
            }
        };

        let db_user = user_result.map_err(|e| {
            error!("Failed to query user: {}", e);
            Status::internal("Failed to query user")
        })?;

        let grpc_user = GUser {
            id: db_user.id,
            email: db_user.email,
        };

        info!("Returning user: {:?}", grpc_user);
        Ok(Response::new(grpc_user))
    }

    async fn create_user(&self, user: DBUser) -> Result<(), anyhow::Error> {
        let mut connection = self.db_connection_manager.get_connection()?;

        diesel::insert_into(users::table)
            .values(&user)
            .execute(&mut connection)
            .map_err(|e| {
                error!("Failed to create user: {}", e);
                anyhow::Error::new(e)
            })?;

        Ok(())
    }
}

module! {
    pub UserManagerModule {
        components = [UserManagerImpl],
        providers = [],
        use DBConnectionManagerModule {
            components = [dyn DBConnectionManager],
            providers = [],
        },
        use ChannelManagerModule {
            components = [dyn ChannelManager],
            providers = [],
        }
    }
}

pub fn build_user_manager_module() -> Arc<UserManagerModule> {
    Arc::new(
        UserManagerModule::builder(
            build_db_connection_manager_module(),
            build_channel_manager_module(),
        )
        .build(),
    )
}
