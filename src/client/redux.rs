use crate::client::redux::actions::Action;
use crate::client::redux::reducers::AppReducer;
use std::cell::RefCell;
use std::rc::Rc;

pub mod actions;
mod reducers;

#[derive(Default)]
pub struct Store {
    state: State,
    app_reducer: AppReducer,
}

pub struct State {
    pub(crate) messages: Rc<RefCell<Vec<String>>>,
}

impl State {
    fn new(messages: Rc<RefCell<Vec<String>>>) -> State {
        State { messages }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            messages: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        State::new(self.messages.clone())
    }
}

impl Store {
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn dispatch(&self, action: Action) -> State {
        self.app_reducer.reduce(action, self.state.clone())
    }
}
