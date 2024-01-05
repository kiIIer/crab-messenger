use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::chats::{
    build_chats_reducer_module, ChatsReducer, ChatsReducerModule,
};
use crate::client::redux::reducers::app::login::ReducersLoginModule;
use crate::client::redux::reducers::app::login::{build_reducers_login_module, LoginReducer};
use crate::client::redux::reducers::app::messages::{
    build_messages_reducer_module, MessagesReducer, MessagesReducerModule,
};
use crate::client::redux::reducers::app::server::{
    build_server_reducer_module, ServerReducer, ServerReducerModule,
};
use crate::client::redux::reducers::app::typing::{
    build_typing_reducer_module, TypingReducer, TypingReducerModule,
};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::client_chat::ChatsState;
use crate::client::redux::state::tab::TabState;
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;

mod login;
mod server;

mod chats;

mod messages;

mod typing;

pub trait AppReducer: Reducer + Interface {}

#[derive(Component)]
#[shaku(interface = AppReducer)]
pub struct AppReducerImpl {
    #[shaku(inject)]
    login_reducer: Arc<dyn LoginReducer>,
    #[shaku(inject)]
    server_reducer: Arc<dyn ServerReducer>,
    #[shaku(inject)]
    chats_reducer: Arc<dyn ChatsReducer>,
    #[shaku(inject)]
    messages_reducer: Arc<dyn MessagesReducer>,
    #[shaku(inject)]
    typing_reducer: Arc<dyn TypingReducer>,
}

impl Reducer for AppReducerImpl {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        if let Action::Input(Event::Key(key)) = action {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                let mut new_state = state.clone();
                new_state.should_exit = true;
                return ReduceResult::Consumed(new_state);
            }
        }
        let login_result =
            self.login_reducer
                .reduce(action, state, dispatch_tx.clone(), handle.clone());

        match login_result {
            ReduceResult::Ignored => {}
            _ => return login_result,
        }

        let server_result =
            self.server_reducer
                .reduce(action, state, dispatch_tx.clone(), handle.clone());

        match server_result {
            ReduceResult::Ignored => {}
            _ => return server_result,
        }

        if state.tab_state == TabState::Chats {
            let reducer_result = match state.chats_state {
                ChatsState::Chats => {
                    self.chats_reducer
                        .reduce(action, state, dispatch_tx.clone(), handle.clone())
                }
                ChatsState::Messages => {
                    self.messages_reducer
                        .reduce(action, state, dispatch_tx.clone(), handle.clone())
                }
                ChatsState::Typing => {
                    self.typing_reducer
                        .reduce(action, state, dispatch_tx, handle)
                }
            };

            match reducer_result {
                ReduceResult::Ignored => {}
                _ => return reducer_result,
            }

            if let Action::Input(Event::Key(key)) = action {
                match key.code {
                    KeyCode::Char('l') => {
                        let mut new_state = state.clone();
                        new_state.chats_state = ChatsState::Messages;
                        return ReduceResult::Consumed(new_state);
                    }
                    KeyCode::Char('h') => {
                        let mut new_state = state.clone();
                        new_state.chats_state = ChatsState::Chats;
                        return ReduceResult::Consumed(new_state);
                    }
                    KeyCode::Char('i') => {
                        let mut new_state = state.clone();
                        new_state.chats_state = ChatsState::Typing;
                        return ReduceResult::Consumed(new_state);
                    }
                    KeyCode::Esc => {
                        let mut new_state = state.clone();
                        new_state.chats_state = ChatsState::Messages;
                        return ReduceResult::Consumed(new_state);
                    }
                    _ => {}
                }
            }
        }

        if let Action::Input(Event::Key(key)) = action {
            if key.code == KeyCode::Char('1') {
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Home;
                return ReduceResult::Consumed(new_state);
            }

            if key.code == KeyCode::Char('0') {
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Login;
                return ReduceResult::Consumed(new_state);
            }

            if key.code == KeyCode::Char('2') {
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Chats;
                new_state.selected_chat = Some(2);
                return ReduceResult::Consumed(new_state);
            }

            if key.code == KeyCode::Char('3') {
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Users;
                return ReduceResult::Consumed(new_state);
            }
        }

        ReduceResult::Ignored
    }
}

impl AppReducer for AppReducerImpl {}

module! {
    pub ReducersAppModule {
        components = [AppReducerImpl],
        providers = [],
        use ReducersLoginModule {
            components = [dyn LoginReducer],
            providers = [],
        },
        use ServerReducerModule {
            components = [dyn ServerReducer],
            providers = [],
        },
        use ChatsReducerModule {
            components = [dyn ChatsReducer],
            providers = [],
        },
        use MessagesReducerModule {
            components = [dyn MessagesReducer],
            providers = [],
        },
        use TypingReducerModule {
            components = [dyn TypingReducer],
            providers = [],
        }
    }
}
pub fn build_reducers_app_module() -> Arc<ReducersAppModule> {
    Arc::new(
        ReducersAppModule::builder(
            build_reducers_login_module(),
            build_server_reducer_module(),
            build_chats_reducer_module(),
            build_messages_reducer_module(),
            build_typing_reducer_module(),
        )
        .build(),
    )
}
