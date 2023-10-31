use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::io::Stdout;
use std::sync::Arc;

pub trait LoginView: Interface + View {}

#[derive(Component)]
#[shaku(interface = LoginView)]
pub struct LoginViewImpl {}

impl LoginView for LoginViewImpl {}

impl View for LoginViewImpl {
    fn draw(
        &self,
        f: &mut Frame<CrosstermBackend<Stdout>>,
        rect: Rect,
        state: State,
    ) -> anyhow::Result<()> {
        let token_text = state
            .auth_state
            .map_or("No token".to_string(), |response| response.access_token);

        let code_text = state.code.unwrap_or("Still working on it ^^'".to_string());

        let uri_text = state
            .link
            .unwrap_or("Still working on it as well ^^'".to_string());

        let p = Paragraph::new(vec![
            Line::from("Login"),
            Line::from(
                "Welcome to the Crab messenger, a terminal messenger written fully in Rust! Please login",
            ),
            Line::from("Login link will soon open in your browser. Please log in there and then proceed in this app."),
            Line::from(format!("If it didn't work, or you know you don't have ui browser, please visit this link: {} , and enter this code: {} on your phone", uri_text, code_text)),
            Line::from(format!("This is your access_token: {}", token_text)),
        ])
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );

        f.render_widget(p, rect);

        Ok(())
    }
}

module! {
    pub ViewLoginModule {
        components = [LoginViewImpl],
        providers = [],
    }
}

pub fn build_view_app_module() -> Arc<ViewLoginModule> {
    Arc::new(ViewLoginModule::builder().build())
}
