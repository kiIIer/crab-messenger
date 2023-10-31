use crate::client::redux::action::Action;
use crate::client::redux::state::State;
use crate::client::redux::store::store_impl::{
    build_store_impl_module, StoreImpl, StoreImplModule, StoreImplParameters,
};
use crossbeam_channel::{Receiver, Sender};
use shaku::{module, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;

mod store_impl;

pub trait Store: Interface {
    fn get_dispatch(&self) -> Sender<Action>;
    fn get_select(&self) -> Receiver<State>;
    fn process(&self, handle: Handle) -> anyhow::Result<()>;
}

module! {
    pub StoreModule {
        components = [],
        providers = [],
        use StoreImplModule {
            components = [dyn Store],
            providers = []
        }
    }
}

pub fn build_store_module() -> Arc<StoreModule> {
    Arc::new(StoreModule::builder(build_store_impl_module()).build())
}
