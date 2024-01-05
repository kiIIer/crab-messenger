use crate::client::redux::state::client_chat::ChatsState;
use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

pub trait ChatView: View + Interface {}

#[derive(Component)]
#[shaku(interface = ChatView)]
pub struct ChatViewImpl {}

impl ChatView for ChatViewImpl {}

impl View for ChatViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let mut list_state = ListState::default();
        list_state.select(state.selected_chat);
        let chats_lock = state.chats.read().unwrap();

        let items: Vec<ListItem> = chats_lock
            .iter()
            .map(|chat| ListItem::new(chat.name.as_str()).style(Style::default().fg(Color::White))) // Set chat text color to white
            .collect();

        let color = match state.chats_state {
            ChatsState::Chats => Color::Red,
            _ => Color::White,
        };

        let chats_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Chats")
                    .style(Style::default().fg(color)),
            )
            .highlight_style(Style::default().bg(Color::Red))
            .highlight_symbol(">> ");

        f.render_stateful_widget(chats_list, rect, &mut list_state);

        Ok(())
    }
}

impl ChatViewImpl {}

module! {
    pub ChatViewModule {
        components = [ChatViewImpl],
        providers = [],
    }
}

pub fn build_chat_view_module() -> Arc<ChatViewModule> {
    Arc::new(ChatViewModule::builder().build())
}
