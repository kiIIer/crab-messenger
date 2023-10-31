use crate::client::redux::reducers::{build_reducer_module, Reducer, ReducersModule};
use shaku::module;
use std::sync::Arc;

pub mod action;
pub mod reducers;
pub mod state;
pub mod store;

module! {
    pub ReduxModule{
        components = [],
        providers = [],
        use ReducersModule{
            components = [],
            providers = [],
        }
    }
}

pub fn build_redux_module() -> Arc<ReduxModule> {
    Arc::new(ReduxModule::builder(build_reducer_module()).build())
}
