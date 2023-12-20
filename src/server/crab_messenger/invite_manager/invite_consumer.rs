use crate::utils::persistence::invite::Invite as DBInvite;
use amqprs::channel::{BasicAckArguments, BasicCancelArguments, Channel, ConsumerMessage};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tonic::Status;
use tracing::field::debug;
use tracing::{debug, error, info, instrument};

pub struct RabbitConsumer {
    tx: mpsc::Sender<Result<crate::utils::messenger::Invite, Status>>,
    queue_name: String,
}

impl RabbitConsumer {
    pub fn new(
        tx: mpsc::Sender<Result<crate::utils::messenger::Invite, Status>>,
        queue_name: String,
    ) -> Self {
        Self { tx, queue_name }
    }

    #[instrument(skip(self, channel, message_rx))]
    pub async fn consume(
        &mut self,
        channel: &Channel,
        tag: String,
        mut message_rx: UnboundedReceiver<ConsumerMessage>,
    ) {
        loop {
            select! {
                    Some(message) = message_rx.recv() => {
                    let deliver = message.deliver.unwrap();
                    let basic_properties = message.basic_properties.unwrap();
                    let content = message.content.unwrap();

                    debug("Sending invite to user");
                    let db_invite: DBInvite = match serde_json::from_slice(&content) {
                        Ok(invite) => invite,
                        Err(e) => {
                            error!("Failed to deserialize invite: {:?}", e);
                            return;
                        }
                    };

                    let grpc_invite = db_invite.into();

                    if let Err(e) = self.tx.send(Ok(grpc_invite)).await {
                        error!("Failed to send invite: {:?}", e);
                        let _ = channel.basic_cancel(BasicCancelArguments::new(&tag)).await.map_err(|err| {
                            error!("Failed to cancel consumer: {:?}", err);
                        });
                    }

                    if let Err(e) = channel
                        .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
                        .await
                    {
                        error!("Failed to acknowledge message: {:?}", e);
                    }

                    if let Err(e) = channel.basic_cancel(BasicCancelArguments::new(&tag)).await {
                        error!("Failed to cancel consumer: {:?}", e);
                    };

                }

                _ = self.tx.closed() => {
                    info!("Client likely disconnected, deleting queue.");
                    let _ = channel.basic_cancel(BasicCancelArguments::new(&tag)).await.map_err(|e| {
                        error!("Failed to cancel consumer: {:?}", e);
                        e
                    });
                    return;
                }
            }
        }
    }
}
