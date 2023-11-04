use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::login::ReducersLoginModule;
use crate::client::redux::reducers::app::login::{build_reducers_login_module, LoginReducer};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;
use crate::client::redux::state::tab::TabState;

mod login;

pub trait AppReducer: Reducer + Interface {}

#[derive(Component)]
#[shaku(interface = AppReducer)]
pub struct AppReducerImpl {
    #[shaku(inject)]
    login_reducer: Arc<dyn LoginReducer>,
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
        let login_result = self
            .login_reducer
            .reduce(action, state, dispatch_tx, handle);

        match login_result {
            ReduceResult::Ignored => {},
            _ => return login_result,
        }

        if let Action::Input(Event::Key(key)) = action{
            if key.code == KeyCode::Char('1'){
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Home;
                return ReduceResult::Consumed(new_state)
            }

            if key.code == KeyCode::Char('0'){
                let mut new_state = state.clone();
                new_state.tab_state = TabState::Login;
                return ReduceResult::Consumed(new_state)
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
        }
    }
}

pub fn build_reducers_app_module() -> Arc<ReducersAppModule> {
    Arc::new(ReducersAppModule::builder(build_reducers_login_module()).build())
}
