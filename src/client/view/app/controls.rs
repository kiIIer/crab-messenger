use crate::client::redux::state::client_chat::ChatsState;
use crate::client::redux::state::tab::TabState;
use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

pub trait ControlsView: View + Interface {}

#[derive(Component)]
#[shaku(interface = ControlsView)]
pub struct ControlsViewImpl {}

impl ControlsView for ControlsViewImpl {}

impl View for ControlsViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let controls_text = match state.tab_state {
            TabState::Chats => match state.chats_state {
                ChatsState::Chats => "| j/k: Select chat | h: Chat select | l: Messages",
                ChatsState::Messages => "| j/k: Select message | i: Insert mode",
                ChatsState::Typing => {
                    "| Type message | Enter: Send | Backspace: Delete | Esc: Exit Insert mode"
                }
                _ => "",
            },
            _ => "",
        };

        let text = format!("Controls: 0-1: Navigate tabs {}", controls_text);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::White));

        f.render_widget(paragraph, rect);

        Ok(())
    }
}

module! {
    pub ViewControlsModule {
        components = [ControlsViewImpl],
        providers = [],
    }
}

pub fn build_view_controls_module() -> Arc<ViewControlsModule> {
    Arc::new(ViewControlsModule::builder().build())
}
