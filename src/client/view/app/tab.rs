use crate::client::redux::state::tab::TabState;
use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::prelude::Span;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::io::Stdout;
use std::sync::Arc;

impl From<TabState> for usize {
    fn from(value: TabState) -> Self {
        match value {
            TabState::Login => 0,
            TabState::Home => 1,
            TabState::Chats => 2,
            TabState::Users => 3,
        }
    }
}

pub trait TabView: Interface + View {}

#[derive(Component)]
#[shaku(interface = TabView)]
pub struct TabViewImpl {
    titles: Vec<String>,
}

impl TabView for TabViewImpl {}

impl View for TabViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let tab_titles = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                Line::from(vec![
                    Span::styled::<String>(format!("{}. ", i), Style::default().fg(Color::Red)),
                    Span::styled(t, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .select(state.tab_state.into())
            .block(
                Block::default()
                    .title("Tabs")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().fg(Color::LightRed));

        f.render_widget(tabs, rect);
        Ok(())
    }
}

module! {
    pub ViewTabModule {
        components = [TabViewImpl],
        providers = [],
    }
}

pub fn build_view_tab_module() -> Arc<ViewTabModule> {
    Arc::new(
        ViewTabModule::builder()
            .with_component_parameters::<TabViewImpl>(TabViewImplParameters {
                titles: vec!["Login".to_string(), "Home".to_string(), "Chats".to_string(), "Users".to_string()],
            })
            .build(),
    )
}
