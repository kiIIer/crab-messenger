use crate::client::redux::state::client_chat::{ChatsState, ClientChatState};
use crate::client::redux::state::tab::TabState;
use crate::utils::auth::AuthState;
use crate::utils::messenger::{Chat, SendMessage, User};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

pub mod client_chat;
pub mod tab;

#[derive(Default, Clone)]
pub struct State {
    pub tab_state: TabState,
    pub auth_state: Option<AuthState>,
    pub code: Option<String>,
    pub link: Option<String>,
    pub messages: Arc<RwLock<Vec<String>>>,
    pub users: Arc<RwLock<Vec<User>>>,
    pub chats: Arc<RwLock<Vec<ClientChatState>>>,
    pub selected_chat: Option<usize>,
    pub chats_state: ChatsState,
    pub should_exit: bool,
    pub send_message_tx: Option<mpsc::Sender<SendMessage>>,
}

// impl State {
//     fn new(
//         poll_response: Option<AuthState>,
//         code: Option<String>,
//         link: Option<String>,
//         messages: Arc<RwLock<Vec<String>>>,
//         should_exit: bool,
//     ) -> Self {
//         Self {
//             auth_state: poll_response,
//             code,
//             link,
//             messages,
//             should_exit,
//         }
//     }
// }
//
// impl Default for State {
//     fn default() -> Self {
//         State {
//             auth_state: None,
//             code: None,
//             link: None,
//             messages: Arc::new(RwLock::new(Vec::new())),
//             should_exit: false,
//         }
//     }
// }
//
// impl Clone for State {
//     fn clone(&self) -> Self {
//         State::new(
//             self.auth_state.clone(),
//             self.code.clone(),
//             self.link.clone(),
//             self.messages.clone(),
//             self.should_exit.clone(),
//         )
//     }
// }
