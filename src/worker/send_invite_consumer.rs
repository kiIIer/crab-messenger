use std::sync::Arc;

use amqprs::channel::{BasicAckArguments, BasicPublishArguments, BasicRejectArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use diesel::{PgConnection, RunQueryDsl};
use tracing::{debug, error, instrument, warn};

use crate::utils::db_connection_manager::DBConnectionManager;
use crate::utils::persistence::invite::{InsertInvite, Invite};
use crate::utils::persistence::schema::invites;
use crate::utils::rabbit_declares::{send_to_error_queue, INVITES_EXCHANGE};

#[derive(Clone)]
pub struct SendInviteConsumer {
    connection_manager: Arc<dyn DBConnectionManager>,
}

#[async_trait]
impl AsyncConsumer for SendInviteConsumer {
    #[instrument(skip(self, channel, content))]
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ) {
        debug!("Received invite");
        if let Err(e) = self.process_invite(channel, &deliver, &content).await {
            error!("Failed to process invite: {:?}", e);
            if let Err(e) = self.reject_message(channel, &deliver, false, content).await {
                error!("Failed to reject message: {:?}", e);
            };
        }
        debug!("Invite processed");
    }
}

impl SendInviteConsumer {
    pub fn new(connection_manager: Arc<dyn DBConnectionManager>) -> Self {
        Self { connection_manager }
    }

    async fn process_invite(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        content: &[u8],
    ) -> Result<(), anyhow::Error> {
        let mut db_connection = self.connection_manager.get_connection()?;
        let send_invite = self.deserialize_message(content)?;
        self.insert_and_publish_message(&mut db_connection, channel, &send_invite, deliver)
            .await?;
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .map_err(|e| anyhow::Error::new(e))?;

        Ok(())
    }

    fn deserialize_message(&self, content: &[u8]) -> Result<InsertInvite, serde_json::Error> {
        let message_str = String::from_utf8_lossy(content);
        serde_json::from_str::<InsertInvite>(&message_str)
    }

    async fn insert_and_publish_message(
        &self,
        db_connection: &mut PgConnection,
        channel: &Channel,
        insert_invite: &InsertInvite,
        deliver: &Deliver,
    ) -> Result<(), anyhow::Error> {
        let invite = self.insert_invite(db_connection, insert_invite)?;
        self.publish_message(channel, &invite, deliver).await?;
        Ok(())
    }

    fn insert_invite(
        &self,
        db_connection: &mut PgConnection,
        insert_invite: &InsertInvite,
    ) -> Result<Invite, diesel::result::Error> {
        diesel::insert_into(invites::table)
            .values(insert_invite)
            .get_result(db_connection)
    }

    async fn publish_message(
        &self,
        channel: &Channel,
        invite: &Invite,
        deliver: &Deliver,
    ) -> Result<(), anyhow::Error> {
        let serialized_message = serde_json::to_string(invite)?;
        channel
            .basic_publish(
                BasicProperties::default(),
                serialized_message.into_bytes(),
                BasicPublishArguments::new(
                    INVITES_EXCHANGE,
                    &format!("{}", invite.invitee_user_id),
                )
                .mandatory(false)
                .immediate(false)
                .finish(),
            )
            .await
            .map_err(|e| anyhow::Error::new(e))?;

        debug!("Invite published successfully");
        Ok(())
    }

    #[instrument(skip(self, channel))]
    async fn reject_message(
        &self,
        channel: &Channel,
        deliver: &Deliver,
        requeue: bool,
        content: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        warn!("Rejecting message");
        send_to_error_queue(channel, content).await?;

        channel
            .basic_reject(BasicRejectArguments::new(deliver.delivery_tag(), requeue))
            .await
            .map_err(|e| {
                error!("Failed to reject message: {:?}", e);
                anyhow::Error::new(e)
            })
    }
}
