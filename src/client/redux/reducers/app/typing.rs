use std::sync::Arc;

use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode};
use shaku::{module, Component, Interface};
use tokio::runtime::Handle;

use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;
use crate::utils::messenger::SendMessage;

pub trait TypingReducer: Reducer + Interface {}

#[derive(Component)]
#[shaku(interface = TypingReducer)]
pub struct TypingReducerImpl {}

impl TypingReducer for TypingReducerImpl {}

impl Reducer for TypingReducerImpl {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        match action {
            Action::Input(Event::Key(event)) => {
                let mut new_state = state.clone();
                let mut state_changed = false;

                {
                    if let Some(selected_chat_index) = new_state.selected_chat {
                        if let Ok(mut chats_lock) = new_state.chats.write() {
                            if let Some(chat) = chats_lock.get_mut(selected_chat_index) {
                                state_changed = match event.code {
                                    KeyCode::Char(c) => {
                                        chat.text.push(c);
                                        true
                                    }
                                    KeyCode::Backspace => chat.text.pop().is_some(),
                                    KeyCode::Enter => {
                                        let text = chat.text.clone();
                                        chat.text.clear();
                                        let send_message = SendMessage {
                                            chat_id: chat.id,
                                            text,
                                        };
                                        dispatch_tx
                                            .send(Action::SendMessage(send_message))
                                            .unwrap();
                                        true
                                    }
                                    _ => false,
                                };
                            }
                        }
                    }
                }

                if state_changed {
                    ReduceResult::Consumed(new_state)
                } else {
                    ReduceResult::Ignored
                }
            }
            _ => ReduceResult::Ignored,
        }
    }
}

module! {
    pub TypingReducerModule {
        components = [TypingReducerImpl],
        providers = []
    }
}

pub fn build_typing_reducer_module() -> Arc<TypingReducerModule> {
    Arc::new(TypingReducerModule::builder().build())
}
