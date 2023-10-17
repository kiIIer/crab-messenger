use crate::client::redux::actions::Action;
use crate::client::redux::State;
use crossterm::event::{Event, KeyCode};

#[derive(Default)]
pub struct AppReducer {}

impl AppReducer {
    pub fn reduce(&self, action: Action, state: State) -> State {
        match action {
            Action::Input(input_event) => match input_event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('w') {
                        state
                            .messages
                            .borrow_mut()
                            .push("You pressed W".to_string())
                    }
                }
                _ => {}
            },
        }
        state
    }
}
