use crate::utils::auth::AuthState;
use crate::client::redux::state::tab::TabState;
use std::sync::{Arc, RwLock};

pub mod tab;

#[derive(Default, Clone)]
pub struct State {
    pub tab_state: TabState,
    pub auth_state: Option<AuthState>,
    pub code: Option<String>,
    pub link: Option<String>,
    pub messages: Arc<RwLock<Vec<String>>>,
    pub should_exit: bool,
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
