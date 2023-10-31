use std::sync::{Arc, RwLock};
use crate::auth::AuthState;

pub mod tab;

pub struct State {
    pub auth_state: Option<AuthState>,
    pub code: Option<String>,
    pub link: Option<String>,
    pub messages: Arc<RwLock<Vec<String>>>,
}

impl State {
    fn new(
        poll_response: Option<AuthState>,
        code: Option<String>,
        link: Option<String>,
        messages: Arc<RwLock<Vec<String>>>,
    ) -> Self {
        Self {
            auth_state: poll_response,
            code,
            link,
            messages,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            auth_state: None,
            code: None,
            link: None,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        State::new(
            self.auth_state.clone(),
            self.code.clone(),
            self.link.clone(),
            self.messages.clone(),
        )
    }
}
