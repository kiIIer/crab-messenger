use crate::client::redux::state::tab::TabState;
use crate::client::redux::state::State;
use crate::client::view::app::chats_container::{
    build_chat_container_view_module, ChatContainerView, ChatContainerViewModule,
};
use crate::client::view::app::controls::{
    build_view_controls_module, ControlsView, ViewControlsModule,
};
use crate::client::view::app::home::{build_view_home_module, HomeView, ViewHomeModule};
use crate::client::view::app::login::{build_view_login_module, LoginView, ViewLoginModule};
use crate::client::view::app::tab::{build_view_tab_module, TabView, ViewTabModule};
use crate::client::view::app::users::{build_user_view_module, UserView, ViewUserModule};
use crate::client::view::View;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::io;
use std::sync::Arc;

mod chats_container;
mod home;
mod login;
mod tab;
mod users;

mod controls;

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

    #[shaku(inject)]
    user: Arc<dyn UserView>,

    #[shaku(inject)]
    chat_container: Arc<dyn ChatContainerView>,

    #[shaku(inject)]
    controls: Arc<dyn ControlsView>,
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
            TabState::Chats => self.chat_container.draw(f, chunks_main[1], state.clone())?,
            TabState::Users => self.user.draw(f, chunks_main[1], state.clone())?,
        }
        self.controls.draw(f, chunks_main[2], state)?;

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
        },
        use ViewUserModule {
            components = [dyn UserView],
            providers = [],
        },
        use ChatContainerViewModule {
            components = [dyn ChatContainerView],
            providers = [],
        },
        use ViewControlsModule {
            components = [dyn ControlsView],
            providers = [],
        },
    }
}

pub fn build_app_view_module() -> Arc<AppViewModule> {
    Arc::new(
        AppViewModule::builder(
            build_view_login_module(),
            build_view_tab_module(),
            build_view_home_module(),
            build_user_view_module(),
            build_chat_container_view_module(),
            build_view_controls_module(),
        )
        .build(),
    )
}
