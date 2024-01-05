use crate::client::redux::state::client_chat::ChatsState;
use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use textwrap::WordSplitter;

pub trait MessagesView: View + Interface {}

#[derive(Component)]
#[shaku(interface = MessagesView)]
pub struct MessagesViewImpl {}

impl MessagesView for MessagesViewImpl {}

impl View for MessagesViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        if let Some(selected_chat_index) = state.selected_chat {
            let chats_lock = state.chats.read().unwrap();
            let users_lock = state.users.read().unwrap();

            if let Some(selected_chat) = chats_lock.get(selected_chat_index) {
                let chat_messages = &selected_chat.messages;

                // Create a ListState and adjust the selected message index
                let mut list_state = ListState::default();
                list_state.select(selected_chat.selected_message);

                // Create ListItems for each message, in reverse order
                let items: Vec<ListItem> = chat_messages
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(index, message)| {
                        let user_email = users_lock
                            .iter()
                            .find(|user| user.id == message.user_id)
                            .map_or("Unknown".to_string(), |user| user.email.clone());

                        let message_number = chat_messages.len() - 1 - index;
                        let full_message =
                            format!("{} [{}]: {}", message_number, user_email, message.text);
                        let width = rect.width as usize - 2;
                        let options = textwrap::Options::new(width);
                        let message_lines = textwrap::wrap(&full_message, options)
                            .iter()
                            .map(|line| Line::from(line.to_string()))
                            .collect::<Vec<Line>>();

                        ListItem::new(message_lines)
                    })
                    .collect();

                let color = match state.chats_state {
                    ChatsState::Messages => Color::Red,
                    _ => Color::White,
                };

                // Create and render the List widget
                let messages_list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Messages")
                            .style(Style::default().fg(color)),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::Red));

                f.render_stateful_widget(messages_list, rect, &mut list_state);
            }
        }

        Ok(())
    }
}

module! {
    pub MessagesViewModule {
        components = [MessagesViewImpl],
        providers = [],
    }
}

pub fn build_messages_view_module() -> Arc<MessagesViewModule> {
    Arc::new(MessagesViewModule::builder().build())
}
