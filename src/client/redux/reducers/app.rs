use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::login::ReducersLoginModule;
use crate::client::redux::reducers::app::login::{build_reducers_login_module, LoginReducer};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;

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
        action: Action,
        state: State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        let login_result = self
            .login_reducer
            .reduce(action, state, dispatch_tx, handle);
        login_result
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
