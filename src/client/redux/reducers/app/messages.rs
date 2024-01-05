use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::app::chats::ChatsReducerImpl;
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode};
use shaku::{module, Component, Interface};
use std::sync::Arc;
use tokio::runtime::Handle;

pub trait MessagesReducer: Interface + Reducer {}

#[derive(Component)]
#[shaku(interface = MessagesReducer)]
pub struct MessagesReducerImpl {}

impl MessagesReducer for MessagesReducerImpl {}

enum MoveDirection {
    Up,
    Down,
    Start,
}

impl Reducer for MessagesReducerImpl {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        match action {
            Action::Input(Event::Key(key_event)) => {
                let mut new_state = state.clone();

                let state_changed = match key_event.code {
                    KeyCode::Char('j') => {
                        self.update_selected_message(&mut new_state, MoveDirection::Down)
                    }
                    KeyCode::Char('k') => {
                        self.update_selected_message(&mut new_state, MoveDirection::Up)
                    }
                    KeyCode::Char('g') => {
                        self.update_selected_message(&mut new_state, MoveDirection::Start)
                    }
                    _ => false,
                };

                if state_changed {
                    dispatch_tx.send(Action::CheckChat).unwrap();
                    return ReduceResult::Consumed(new_state);
                } else {
                    return ReduceResult::Ignored;
                }
            }
            Action::CheckChat => {
                if let Some(current_chat) = state.selected_chat {
                    if let Ok(chats_lock) = state.chats.read() {
                        if let Some(chat) = chats_lock.get(current_chat) {
                            if chat.selected_message.is_none()
                                || chat.selected_message.unwrap() == chat.messages.len() - 1
                            {
                                dispatch_tx.send(Action::LoadMessages).unwrap();
                                return ReduceResult::ConsumedButKindaNot;
                            }
                        }
                    }
                }

                ReduceResult::Ignored
            }

            _ => ReduceResult::Ignored,
        }
    }
}

impl MessagesReducerImpl {
    fn update_selected_message(&self, state: &mut State, direction: MoveDirection) -> bool {
        if let Some(selected_chat_index) = state.selected_chat {
            if let Ok(mut chats_lock) = state.chats.write() {
                if let Some(chat) = chats_lock.get_mut(selected_chat_index) {
                    let max_message_index = chat.messages.len().saturating_sub(1);

                    match direction {
                        MoveDirection::Up => {
                            if let Some(selected_message) = chat.selected_message {
                                if selected_message > 0 {
                                    chat.selected_message = Some(selected_message - 1);
                                    return true;
                                }
                            }
                        }
                        MoveDirection::Down => {
                            if let Some(selected_message) = chat.selected_message {
                                if selected_message < max_message_index {
                                    chat.selected_message = Some(selected_message + 1);
                                    return true;
                                }
                            }
                        }
                        MoveDirection::Start => {
                            chat.selected_message = Some(0);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

module! {
    pub MessagesReducerModule {
        components = [MessagesReducerImpl],
        providers = []
    }
}

pub fn build_messages_reducer_module() -> Arc<MessagesReducerModule> {
    Arc::new(MessagesReducerModule::builder().build())
}
