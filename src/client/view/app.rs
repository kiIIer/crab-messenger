use crate::client::redux::state::tab::TabState;
use crate::client::redux::state::State;
use crate::client::view::app::home::{build_view_home_module, HomeView, ViewHomeModule};
use crate::client::view::app::login::{build_view_login_module, LoginView, ViewLoginModule};
use crate::client::view::app::tab::{build_view_tab_module, TabView, ViewTabModule};
use crate::client::view::View;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::io;
use std::sync::Arc;

mod home;
mod login;
mod tab;

pub trait AppView: View + Interface {}

#[derive(Component)]
#[shaku(interface = AppView)]
pub struct AppViewImpl {
    #[shaku(inject)]
    login: Arc<dyn LoginView>,

    #[shaku(inject)]
    tab: Arc<dyn TabView>,

    #[shaku(inject)]
    home: Arc<dyn HomeView>,
}

impl AppView for AppViewImpl {}

impl View for AppViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
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

        self.tab.draw(f, chunks_main[0], state.clone())?;
        match state.tab_state {
            TabState::Login => self.login.draw(f, chunks_main[1], state.clone())?,
            TabState::Home => self.home.draw(f, chunks_main[1], state.clone())?,
        }

        Ok(())
    }
}

module! {
    pub AppViewModule {
        components = [AppViewImpl],
        providers = [],
        use ViewLoginModule {
            components = [dyn LoginView],
            providers = [],
        },
        use ViewTabModule {
            components = [dyn TabView],
            providers = [],
        },
        use ViewHomeModule {
            components = [dyn HomeView],
            providers = [],
        }
    }
}

pub fn build_app_view_module() -> Arc<AppViewModule> {
    Arc::new(
        AppViewModule::builder(
            build_view_login_module(),
            build_view_tab_module(),
            build_view_home_module(),
        )
        .build(),
    )
}
