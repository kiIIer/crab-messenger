use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::ReducersAppModule;
use crate::client::redux::reducers::app::{build_reducers_app_module, AppReducer};
use crate::client::redux::state::State;
use crate::client::redux::store::Store;
use crossbeam_channel::{Receiver, Sender};
use shaku::{module, Component};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle;

#[derive(Component)]
#[shaku(interface = Store)]
pub struct StoreImpl {
    dispatch_tx: Sender<Action>,
    dispatch_rc: Receiver<Action>,
    select_tx: Sender<State>,
    select_rc: Receiver<State>,

    state: State,

    #[shaku(inject)]
    app_reducer: Arc<dyn AppReducer>,
}

impl Store for StoreImpl {
    fn get_dispatch(&self) -> Sender<Action> {
        self.dispatch_tx.clone()
    }

    fn get_select(&self) -> Receiver<State> {
        self.select_rc.clone()
    }

    fn process(&self, handle: Handle) -> anyhow::Result<()> {
        let dispatch_rc = self.dispatch_rc.clone();
        let app_reducer = self.app_reducer.clone();
        let dispatch_tx = self.dispatch_tx.clone();
        let select_tx = self.select_tx.clone();
        self.select_tx.send(self.state.clone())?;
        let mut state = self.state.clone();
        thread::spawn(move || {
            while let Ok(action) = dispatch_rc.recv() {
                let reduce_result =
                    app_reducer.reduce(action, state.clone(), dispatch_tx.clone(), handle.clone());

                if let ReduceResult::Consumed(new_state) = reduce_result {
                    state = new_state.clone();
                }
                
                select_tx.send(state.clone()).unwrap();
            }
        });
        Ok(())
    }
}

module! {
    pub StoreImplModule {
        components = [StoreImpl],
        providers = [],
        use ReducersAppModule {
            components = [dyn AppReducer],
            providers = [],
        }
    }
}

pub fn build_store_impl_module() -> Arc<StoreImplModule> {
    let (dispatch_tx, dispatch_rc) = crossbeam_channel::unbounded::<Action>();
    let (select_tx, select_rc) = crossbeam_channel::unbounded::<State>();
    Arc::new(
        StoreImplModule::builder(build_reducers_app_module())
            .with_component_parameters::<StoreImpl>(StoreImplParameters {
                dispatch_tx,
                dispatch_rc,
                select_tx,
                select_rc,
                state: State::default(),
            })
            .build(),
    )
}
