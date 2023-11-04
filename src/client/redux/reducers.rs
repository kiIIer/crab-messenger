use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::build_reducers_app_module;
use crate::client::redux::reducers::app::{AppReducer, ReducersAppModule};
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use shaku::{module, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;

pub mod app;

pub trait Reducer: Send + Sync {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult;
}

module! {
    pub ReducersModule {
        components = [],
        providers = [],
        use ReducersAppModule {
            components = [dyn AppReducer],
            providers = [],
        },
    }
}

pub fn build_reducer_module() -> Arc<ReducersModule> {
    Arc::new(ReducersModule::builder(build_reducers_app_module()).build())
}
