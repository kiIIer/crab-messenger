use crate::client::redux::state::State;
use crate::client::view::app::login::ViewLoginModule;
use crate::client::view::app::login::{build_view_app_module, LoginView};
use crate::client::view::View;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::io;
use std::sync::Arc;

mod login;

pub trait AppView: View + Interface {}

#[derive(Component)]
#[shaku(interface = AppView)]
pub struct AppViewImpl {
    #[shaku(inject)]
    login: Arc<dyn LoginView>,
}

impl AppView for AppViewImpl {}

impl View for AppViewImpl {
    fn draw(
        &self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        rect: Rect,
        state: State,
    ) -> anyhow::Result<()> {
        let chunks_main = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(rect);

        self.login.draw(f, chunks_main[1], state)?;

        Ok(())
    }
}

module! {
    pub AppViewModule {
        components = [AppViewImpl],
        providers = [],
        use ViewLoginModule {
            components = [dyn LoginView],
            providers = []
        }
    }
}

pub fn build_app_view_module() -> Arc<AppViewModule> {
    Arc::new(AppViewModule::builder(build_view_app_module()).build())
}
