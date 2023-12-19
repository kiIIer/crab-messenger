use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::{module, Component, Interface};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::utils::messenger::{Invite, SendInviteRequest, SendInviteResponse};
use crate::utils::rabbit_channel_manager::{
    build_channel_manager_module, ChannelManager, ChannelManagerModule,
};
use crate::utils::rabbit_declares::{declare_send_invite_exchange, SEND_INVITE_EXCHANGE};

#[async_trait]
pub trait InviteManager: Interface {
    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status>;
}

#[derive(Component)]
#[shaku(interface = InviteManager)]
pub struct InviteManagerImpl {
    #[shaku(inject)]
    channel_manager: Arc<dyn ChannelManager>,
}

#[derive(Serialize, Deserialize)]
struct RabbitCreateInvite {
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub chat_id: i32,
}

#[async_trait]
impl InviteManager for InviteManagerImpl {
    async fn send_invite(
        &self,
        request: Request<SendInviteRequest>,
    ) -> Result<Response<SendInviteResponse>, Status> {
        info!("Sending invite");

        let metadata = request.metadata();

        let user_id = metadata
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let invite_request = request.into_inner();
        let rabbit_invite = RabbitCreateInvite {
            inviter_user_id: user_id,
            invitee_user_id: invite_request.user_id,
            chat_id: invite_request.chat_id,
        };

        let serialized_message = serde_json::to_string(&rabbit_invite).map_err(|e| {
            error!("Failed to serialize message: {:?}", e);
            Status::internal("Failed to serialize message")
        })?;

        let channel = self.channel_manager.get_channel().await.map_err(|e| {
            error!("Failed to get channel: {:?}", e);
            Status::internal("Failed to get channel")
        })?;

        declare_send_invite_exchange(&channel).await.map_err(|e| {
            error!("Failed to declare exchange: {:?}", e);
            Status::internal("Failed to declare exchange")
        })?;

        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(SEND_INVITE_EXCHANGE, "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish message: {:?}", e);
                Status::internal("Failed to publish message")
            })?;

        Ok(Response::new(SendInviteResponse { success: true }))
    }
}

module! {
    pub InviteManagerModule {
        components = [InviteManagerImpl],
        providers = [],
        use ChannelManagerModule {
            components = [dyn ChannelManager],
            providers = []
        }
    }
}

pub fn build_invite_manager_module() -> Arc<InviteManagerModule> {
    Arc::new(InviteManagerModule::builder(build_channel_manager_module()).build())
}
