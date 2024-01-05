use crate::utils::messenger::{Chat, Message};

#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub enum ChatsState {
    Chats,
    Messages,
    Typing,
}

impl Default for ChatsState {
    fn default() -> Self {
        ChatsState::Chats
    }
}

#[derive(Clone, Debug)]
pub struct ClientChatState {
    pub id: i32,
    pub name: String,
    pub selected_message: Option<usize>,
    pub messages: Vec<Message>,
    pub text: String,
}

impl ClientChatState {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            selected_message: None,
            messages: Vec::new(),
            text: String::new(),
        }
    }
}

impl From<&Chat> for ClientChatState {
    fn from(chat: &Chat) -> Self {
        Self::new(chat.id, chat.name.clone())
    }
}
