use std::sync::Arc;

use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode};
use shaku::{module, Component, Interface};
use tokio::runtime::Handle;

use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;

pub trait ChatsReducer: Interface + Reducer {}

#[derive(Component)]
#[shaku(interface = ChatsReducer)]
pub struct ChatsReducerImpl {}

impl ChatsReducer for ChatsReducerImpl {}

impl Reducer for ChatsReducerImpl {
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
                    KeyCode::Char('j') => self.update_selected_chat(&mut new_state, true),
                    KeyCode::Char('k') => self.update_selected_chat(&mut new_state, false),
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

impl ChatsReducerImpl {
    fn update_selected_chat(&self, state: &mut State, move_down: bool) -> bool {
        let chats_lock = state.chats.read().unwrap();
        if let Some(selected_chat) = state.selected_chat.as_mut() {
            let max_id = chats_lock.len().saturating_sub(1);
            if move_down && *selected_chat < max_id {
                *selected_chat += 1;
                true
            } else if !move_down && *selected_chat > 0 {
                *selected_chat -= 1;
                true
            } else {
                false
            }
        } else {
            if !chats_lock.is_empty() {
                state.selected_chat = Some(0);
                true
            } else {
                false
            }
        }
    }
}

module! {
    pub ChatsReducerModule {
        components = [ChatsReducerImpl],
        providers = [],
    }
}

pub fn build_chats_reducer_module() -> Arc<ChatsReducerModule> {
    Arc::new(ChatsReducerModule::builder().build())
}
