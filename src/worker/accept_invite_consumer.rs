use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, BasicPublishArguments, BasicRejectArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::PgConnection;
use tonic::Status;
use tracing::{debug, error, info, instrument};

use crate::utils::db_connection_manager::DBConnectionManager;
use crate::utils::persistence::invite::Invite;
use crate::utils::persistence::schema::{invites, users_chats};
use crate::utils::persistence::users_chats::UsersChats;
use crate::utils::rabbit_declares::{
    chat_connect_exchange_name, declare_chat_connect_exchange, CHAT_CONNECT_EXCHANGE,
    INVITES_EXCHANGE,
};
use crate::utils::rabbit_types::RabbitInviteAccept;

#[derive(Clone)]
pub struct AcceptInviteConsumer {
    connection_manager: Arc<dyn DBConnectionManager>,
}

#[async_trait]
impl AsyncConsumer for AcceptInviteConsumer {
    #[instrument(skip(self, channel, content))]
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        debug!("Accepted invite");
        if let Err(e) = self.process_invite(channel, &deliver, &content).await {
            error!("Failed to process invite: {:?}", e);
            if let Err(e) = self.reject_message(channel, &deliver, false, content).await {
                error!("Failed to reject message: {:?}", e);
            };
        }
        debug!("Invite processed");
    }
}

impl AcceptInviteConsumer {
    pub fn new(connection_manager: Arc<dyn DBConnectionManager>) -> Self {
        Self { connection_manager }
    }

    #[instrument(skip(self, channel, deliver, content))]
    async fn process_invite(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        content: &[u8],
    ) -> Result<(), anyhow::Error> {
        let mut db_connection = self.connection_manager.get_connection()?;
        let accept_invite = self.deserialize_message(content)?;
        let chat_id = self
            .accept_invite(&mut db_connection, &accept_invite)
            .await
            .map_err(|e| {
                error!("Failed to accept invite: {:?}", e);
                Status::internal("Failed to accept invite")
            })?;

        info!("Accepted invite to chat {}", chat_id);
        self.notify_chats(channel, chat_id, &accept_invite.user_id)
            .await
            .map_err(|e| {
                error!("Failed to notify chats_container: {:?}", e);
                Status::internal("Failed to notify chats_container")
            })?;

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .map_err(|e| anyhow::Error::new(e))?;
        Ok(())
    }

    #[instrument(skip(self, db_connection, invite_accept))]
    async fn accept_invite(
        &self,
        db_connection: &mut PgConnection,
        invite_accept: &RabbitInviteAccept,
    ) -> Result<i32, anyhow::Error> {
        let invite = invites::table
            .filter(invites::id.eq(invite_accept.invite_id))
            .first::<Invite>(db_connection)?;

        info!("Accepted invite: {:?}", invite);
        let user_chat = UsersChats {
            user_id: invite.invitee_user_id,
            chat_id: invite.chat_id,
        };

        diesel::insert_into(users_chats::table)
            .values(user_chat)
            .execute(db_connection)
            .map_err(|e| {
                error!("Failed to insert user_chat: {:?}", e);
                anyhow::Error::new(e)
            })?;

        diesel::delete(invites::table)
            .filter(invites::chat_id.eq(invite.chat_id))
            .filter(invites::invitee_user_id.eq(&invite_accept.user_id))
            .execute(db_connection)
            .map_err(|e| {
                error!("Failed to delete invite: {:?}", e);
                Status::internal("Failed to delete invite")
            })?;
        Ok(invite.chat_id)
    }

    #[instrument(skip(self, channel))]
    async fn notify_chats(
        &self,
        channel: &Channel,
        chat_id: i32,
        user_id: &str,
    ) -> Result<(), anyhow::Error> {
        declare_chat_connect_exchange(channel, user_id)
            .await
            .map_err(|e| {
                error!("Failed to declare exchange: {:?}", e);
                anyhow::Error::new(e)
            })?;

        channel
            .basic_publish(
                BasicProperties::default(),
                chat_id.to_string().into_bytes(),
                BasicPublishArguments::new(&chat_connect_exchange_name(&user_id), "")
                    .mandatory(false)
                    .immediate(false)
                    .finish(),
            )
            .await
            .map_err(|e| anyhow::Error::new(e))?;

        debug!("Connect command sent successfully");
        Ok(())
    }

    #[instrument(skip(self, content))]
    fn deserialize_message(&self, content: &[u8]) -> Result<RabbitInviteAccept, anyhow::Error> {
        let invite_accept: RabbitInviteAccept = serde_json::from_slice(content).map_err(|e| {
            error!("Failed to deserialize invite: {:?}", e);
            anyhow::Error::new(e)
        })?;
        Ok(invite_accept)
    }

    #[instrument(skip(self, channel, deliver, requeue, content))]
    async fn reject_message(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        requeue: bool,
        content: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        channel
            .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), requeue))
            .await
            .map_err(|e| {
                error!("Failed to reject message: {:?}", e);
                anyhow::Error::new(e)
            })?;
        Ok(())
    }
}
