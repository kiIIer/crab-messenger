use crate::client::redux::state::State;
use crate::utils::auth::{AuthState, StartFlowResponse};
use crate::utils::messenger::{Chat, Message, SendMessage, User};
use crossterm::event::Event;

#[derive(Clone)]
pub enum Action {
    Input(Event),
    StartLogin,
    Login(StartFlowResponse),
    LoginSuccess(AuthState),
    Init,
    Tick,
    LoadUsers,
    LoadUsersSuccess(Vec<User>),
    LoadChats,
    LoadChatsSuccess(Vec<Chat>),
    CheckChat,
    LoadMessages,
    LoadMessagesSuccess(i32, Vec<Message>),
    SetupMessagesStream,
    ReceivedMessage(Message),
    SendMessage(SendMessage),
}

pub enum ReduceResult {
    Consumed(State),
    ConsumedButKindaNot,
    Ignored,
}
