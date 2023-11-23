use crate::messenger::messenger_server::Messenger;
use crate::messenger::{GetMessages, Message as MMessage, Messages, SendMessage};
use crate::server::persistence::message::Message;
use crate::server::persistence::schema::messages;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{PgConnection, RunQueryDsl};
use dotenv::dotenv;
use futures_core::Stream;
use shaku::{module, Component, Interface};
use std::env;
use std::pin::Pin;
use std::sync::Arc;
use tokio::join;
use tonic::{Request, Response, Status, Streaming};

pub trait CrabMessenger: Interface + Messenger {}

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<MMessage, Status>> + Send>>;

#[derive(Component)]
#[shaku(interface = CrabMessenger<ChatStream = ResponseStream>)]
pub struct CrabMessengerImpl {
    pub(crate) pool: Pool<ConnectionManager<PgConnection>>,
}

impl CrabMessenger for CrabMessengerImpl {}

#[async_trait]
impl Messenger for CrabMessengerImpl {
    type ChatStream = ResponseStream;

    async fn chat(
        &self,
        request: Request<Streaming<SendMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        todo!()
    }

    async fn get_messages(
        &self,
        request: Request<GetMessages>,
    ) -> Result<Response<Messages>, Status> {
        let mut connection = self.pool.get().map_err(|e| {
            Status::internal(format!("Failed to get DB connection from pool: {}", e))
        })?;

        let get_messages_req = request.into_inner();
        let chat_id_filter = get_messages_req.chat_id;
        let created_before_filter = get_messages_req.created_before.unwrap(); // assuming it's present

        let created_before_naive = chrono::NaiveDateTime::from_timestamp_opt(
            created_before_filter.seconds,
            created_before_filter.nanos as u32,
        )
        .unwrap();

        let message_results = messages::table
            .filter(messages::chat_id.eq(chat_id_filter))
            .filter(messages::created_at.lt(created_before_naive))
            .load::<Message>(&mut connection)
            .map_err(|e| Status::internal(format!("Failed to query messages: {}", e)))?;

        let proto_messages: Vec<_> = message_results.into_iter().map(Into::into).collect();

        let response = Messages {
            messages: proto_messages,
        };

        Ok(Response::new(response))
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
    }
}

pub fn build_crab_messenger_module() -> Arc<CrabMessengerModule> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).unwrap();
    Arc::new(
        CrabMessengerModule::builder()
            .with_component_parameters::<CrabMessengerImpl>(CrabMessengerImplParameters { pool })
            .build(),
    )
}
