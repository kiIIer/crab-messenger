use crate::client::redux::state::State;
use crate::client::view::View;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

pub trait UserView: Interface + View {}

#[derive(Component)]
#[shaku(interface = UserView)]
pub struct UserViewImpl {}

impl UserView for UserViewImpl {}

impl View for UserViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let users_lock = state.users.read().unwrap();

        let users = users_lock
            .iter()
            .map(|user| Line::from(Line::from(user.email.clone())))
            .collect::<Vec<_>>();

        let p = Paragraph::new(users)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Users")
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );

        f.render_widget(p, rect);

        Ok(())
    }
}

module! {
    pub ViewUserModule {
        components = [UserViewImpl],
        providers = [],
    }
}

pub fn build_user_view_module() -> Arc<ViewUserModule> {
    Arc::new(ViewUserModule::builder().build())
}
