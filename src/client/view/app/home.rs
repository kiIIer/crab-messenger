use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::Rect;
use ratatui::prelude::{Alignment, Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

pub trait HomeView: View + Interface {}

#[derive(Component)]
#[shaku(interface = HomeView)]
pub struct HomeViewImpl {}

impl View for HomeViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let welcome_text = vec![
            Line::from("Welcome to Crab Messenger! 🦀"),
            Line::from(""),
            Line::from(
                "Crab Messenger is a terminal-based messaging platform built entirely in Rust.",
            ),
            Line::from("Experience the charm of chatting right from your terminal!"),
            Line::from(""),
            Line::from("We're delighted to have you on board!"),
            Line::from(""),
            Line::from(
                "Get started by selecting an option from the menu or using available commands.",
            ),
        ];

        let p = Paragraph::new(welcome_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Home")
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );

        f.render_widget(p, rect);

        Ok(())
    }
}

impl HomeView for HomeViewImpl {}

module! {
    pub ViewHomeModule {
        components = [HomeViewImpl],
        providers = [],
    }
}

pub fn build_view_home_module() -> Arc<ViewHomeModule> {
    Arc::new(ViewHomeModule::builder().build())
}
