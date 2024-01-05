use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Error;
use crossbeam_channel::Sender;
use futures::stream::StreamExt;
use shaku::{module, Component, Interface};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::Request;

use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::client_chat::ClientChatState;
use crate::client::redux::state::State;
use crate::utils::messenger::messenger_client::MessengerClient;
use crate::utils::messenger::{
    GetMessagesRequest, GetRelatedUsersRequest, GetUserChatsRequest, SendMessage,
};

pub trait ServerReducer: Reducer + Interface {}

#[derive(Component)]
#[shaku(interface = ServerReducer)]
pub struct ServerReducerImpl {}

impl ServerReducer for ServerReducerImpl {}

impl Reducer for ServerReducerImpl {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        match action {
            Action::Init => {
                dispatch_tx.send(Action::LoadUsers).unwrap();
                dispatch_tx.send(Action::LoadChats).unwrap();
                dispatch_tx.send(Action::CheckChat).unwrap();
                dispatch_tx.send(Action::SetupMessagesStream).unwrap();

                ReduceResult::ConsumedButKindaNot
            }
            Action::LoadUsers => {
                let state = state.clone();
                handle.spawn(async move {
                    let server_address = dotenv!("SERVER_ADDRESS");
                    let mut client = MessengerClient::connect(server_address)
                        .await
                        .expect("Couldn't connect to server");
                    let mut request = tonic::Request::new(GetRelatedUsersRequest {});
                    let auth_token = state.auth_state.unwrap().access_token;

                    let token_metadata =
                        MetadataValue::from_str(&format!("{}", auth_token)).unwrap();
                    request
                        .metadata_mut()
                        .insert("authorization", token_metadata);

                    let response = client.get_related_users(request).await.unwrap();
                    let users = response.into_inner().users;

                    dispatch_tx.send(Action::LoadUsersSuccess(users)).unwrap();
                });

                ReduceResult::ConsumedButKindaNot
            }
            Action::LoadUsersSuccess(users) => {
                let mut new_state = state.clone();

                {
                    let mut users_lock = new_state.users.write().unwrap();
                    users_lock.extend(users.iter().cloned().map(|mut u| {
                        u.email = "That's confidential".to_string();
                        u
                    }));
                }
                ReduceResult::Consumed(new_state)
            }
            Action::LoadChats => {
                let state = state.clone();
                handle.spawn(async move {
                    let server_address = dotenv!("SERVER_ADDRESS");
                    let mut client = MessengerClient::connect(server_address)
                        .await
                        .expect("Couldn't connect to server");
                    let mut request = tonic::Request::new(GetUserChatsRequest {});
                    let auth_token = state.auth_state.unwrap().access_token;

                    let token_metadata =
                        MetadataValue::from_str(&format!("{}", auth_token)).unwrap();
                    request
                        .metadata_mut()
                        .insert("authorization", token_metadata);

                    let response = client.get_user_chats(request).await.unwrap();
                    let chats = response.into_inner().chats;

                    dispatch_tx.send(Action::LoadChatsSuccess(chats)).unwrap();
                });
                ReduceResult::ConsumedButKindaNot
            }
            Action::LoadChatsSuccess(chats) => {
                let mut new_state = state.clone();

                {
                    let mut chats_lock = new_state.chats.write().unwrap();
                    let client_chats: Vec<ClientChatState> =
                        chats.iter().map(|chat| chat.into()).collect();

                    chats_lock.extend(client_chats);
                }
                ReduceResult::Consumed(new_state)
            }
            Action::LoadMessages => {
                if let Some(selected_chat) = state.selected_chat {
                    if let Ok(chats_lock) = state.chats.read() {
                        if let Some(chat) = chats_lock.get(selected_chat) {
                            let chat_id = chat.id;
                            let now = SystemTime::now();
                            let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
                            let timestamp = prost_types::Timestamp {
                                seconds: since_the_epoch.as_secs() as i64,
                                nanos: since_the_epoch.subsec_nanos() as i32,
                            };

                            let timestamp = if chat.messages.is_empty() {
                                timestamp
                            } else {
                                match chat.messages.last() {
                                    Some(message) => {
                                        let created_at = message.created_at.clone();
                                        match created_at {
                                            Some(created_at) => created_at,
                                            None => timestamp,
                                        }
                                    }
                                    None => timestamp,
                                }
                            };

                            let mut request = tonic::Request::new(GetMessagesRequest {
                                chat_id,
                                created_before: Some(timestamp),
                            });

                            let server_address = dotenv!("SERVER_ADDRESS");
                            let auth_token = state.auth_state.clone().unwrap().access_token;

                            let token_metadata =
                                MetadataValue::from_str(&format!("{}", auth_token)).unwrap();
                            request
                                .metadata_mut()
                                .insert("authorization", token_metadata);

                            handle.spawn(async move {
                                let mut client = MessengerClient::connect(server_address)
                                    .await
                                    .expect("Couldn't connect to server");

                                let response = client.get_messages(request).await.unwrap();
                                let messages = response.into_inner().messages;
                                let action = Action::LoadMessagesSuccess(chat_id, messages);
                                dispatch_tx.send(action).unwrap();
                            });

                            return ReduceResult::ConsumedButKindaNot;
                        }
                    }
                }

                ReduceResult::Ignored
            }
            Action::LoadMessagesSuccess(chat_id, messages) => {
                let mut new_state = state.clone();

                if let Ok(mut chats_lock) = new_state.chats.write() {
                    if let Some(chat) = chats_lock.iter_mut().find(|c| c.id == *chat_id) {
                        let existing_message_ids: HashSet<_> =
                            chat.messages.iter().map(|m| m.id).collect();

                        chat.messages.extend(
                            messages
                                .iter()
                                .cloned()
                                .filter(|m| !existing_message_ids.contains(&m.id)),
                        );

                        chat.messages.sort_by(|a, b| {
                            match (&a.created_at, &b.created_at) {
                                (Some(a_created_at), Some(b_created_at)) => {
                                    a_created_at.seconds.cmp(&b_created_at.seconds)
                                }
                                (Some(_), None) => std::cmp::Ordering::Greater, // Assume a is greater if b is None
                                (None, Some(_)) => std::cmp::Ordering::Less, // Assume a is less if a is None
                                (None, None) => std::cmp::Ordering::Equal, // Equal if both are None
                            }
                        });

                        chat.selected_message = Some(0);
                    }
                }

                ReduceResult::Consumed(new_state)
            }
            Action::SetupMessagesStream => {
                let mut new_state = state.clone();
                let (tx, rx) = mpsc::channel(4);
                new_state.send_message_tx = Some(tx);

                let auth_token = state.auth_state.clone().unwrap().access_token;

                let token_metadata = MetadataValue::from_str(&format!("{}", auth_token))
                    .map_err(|e| Error::new(e))
                    .unwrap();
                let mut request = Request::new(ReceiverStream::new(rx));
                request
                    .metadata_mut()
                    .insert("authorization", token_metadata);

                handle.spawn(async move {
                    let mut client = MessengerClient::connect(dotenv!("SERVER_ADDRESS"))
                        .await
                        .unwrap();
                    let response_stream = client
                        .chat(request)
                        .await
                        .expect("Failed to start chat")
                        .into_inner();
                    response_stream
                        .for_each(|message| async {
                            match message {
                                Ok(msg) => dispatch_tx
                                    .clone()
                                    .send(Action::ReceivedMessage(msg))
                                    .unwrap(),
                                Err(e) => eprintln!("Error: {:?}", e),
                            }
                        })
                        .await;
                });

                ReduceResult::Consumed(new_state)
            }
            Action::ReceivedMessage(message) => {
                let mut new_state = state.clone();

                if let Ok(mut chats_lock) = new_state.chats.write() {
                    if let Some(chat) = chats_lock.iter_mut().find(|c| c.id == message.chat_id) {
                        let existing_message_ids: HashSet<_> =
                            chat.messages.iter().map(|m| m.id).collect();

                        if !existing_message_ids.contains(&message.id) {
                            chat.messages.push(message.clone());
                            chat.messages
                                .sort_by(|a, b| match (&a.created_at, &b.created_at) {
                                    (Some(a_created_at), Some(b_created_at)) => {
                                        a_created_at.seconds.cmp(&b_created_at.seconds)
                                    }
                                    (Some(_), None) => std::cmp::Ordering::Greater,
                                    (None, Some(_)) => std::cmp::Ordering::Less,
                                    (None, None) => std::cmp::Ordering::Equal,
                                });

                            // Increment the selected_message if it's not None
                            if let Some(selected_message) = chat.selected_message {
                                if selected_message != 0 {
                                    chat.selected_message = Some(selected_message + 1);
                                }
                            }
                        }
                    }
                }

                ReduceResult::Consumed(new_state)
            }
            Action::SendMessage(send_message) => {
                let mut new_state = state.clone();

                if let Some(tx) = new_state.send_message_tx.clone() {
                    let send_message_clone = send_message.clone();
                    handle.spawn(async move {
                        tx.send(send_message_clone).await.unwrap();
                    });
                }

                ReduceResult::ConsumedButKindaNot
            }

            _ => ReduceResult::Ignored,
        }
    }
}

module! {
    pub ServerReducerModule {
        components = [ServerReducerImpl],
        providers = []
    }
}

pub fn build_server_reducer_module() -> Arc<ServerReducerModule> {
    Arc::new(ServerReducerModule::builder().build())
}
