use crate::client::redux::action::Action;
use crate::client::redux::store::Store;
use crossterm::event;
use crossterm::event::Event::Key;
use crossterm::event::{Event, KeyEventKind};
use shaku::{module, Component, Interface};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub trait Input: Interface {
    fn process(self: Arc<Self>, store: Arc<dyn Store>);
}

#[derive(Component)]
#[shaku(interface = Input)]
pub struct InputImpl {
    dur: Duration,
}

impl InputImpl {
    fn poll(&self) -> anyhow::Result<Option<Event>> {
        if event::poll(self.dur)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

impl Input for InputImpl {
    fn process(self: Arc<Self>, store: Arc<dyn Store>) {
        let dispatch = store.get_dispatch();

        thread::spawn(move || loop {
            if let Some(e) = self.poll().expect("Couldn't poll") {
                if let Key(key) = e {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                }

                dispatch.send(Action::Input(e)).expect("Couldn't send");
            }
        });
    }
}

module! {
    pub InputModule{
        components = [InputImpl],
        providers = [],
    }
}

pub fn build_input_module() -> Arc<InputModule> {
    Arc::new(
        InputModule::builder()
            .with_component_parameters::<InputImpl>(InputImplParameters {
                dur: Duration::from_millis(100),
            })
            .build(),
    )
}
