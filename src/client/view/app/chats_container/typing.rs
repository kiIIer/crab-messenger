use crate::client::redux::state::client_chat::ChatsState;
use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

pub trait TypingView: View + Interface {}

#[derive(Component)]
#[shaku(interface = TypingView)]
pub struct TypingViewImpl {}

impl TypingView for TypingViewImpl {}

impl View for TypingViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        if let Some(selected_chat_index) = state.selected_chat {
            let chats_lock = state.chats.read().unwrap();

            if let Some(chat) = chats_lock.get(selected_chat_index) {
                let color = match state.chats_state {
                    ChatsState::Typing => Color::Red,
                    _ => Color::White,
                };

                // Convert the text to a vector of chars
                let chat_chars: Vec<char> = chat.text.chars().collect();
                let area_width = rect.width.saturating_sub(3) as usize;

                // Determine the starting index based on the width and length of chat_chars
                let start_index = if chat_chars.len() > area_width {
                    chat_chars.len() - area_width
                } else {
                    0
                };

                // Reconstruct the string to display from the character vector
                let display_text: String = chat_chars[start_index..].iter().collect();

                let text = Paragraph::new(display_text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Typing")
                            .style(Style::default().fg(color)),
                    )
                    .style(Style::default().fg(Color::White));

                f.render_widget(text, rect);
            }
        }

        Ok(())
    }
}

module! {
    pub TypingViewModule {
        components = [TypingViewImpl],
        providers = [],
    }
}

pub fn build_typing_view_module() -> Arc<TypingViewModule> {
    Arc::new(TypingViewModule::builder().build())
}
