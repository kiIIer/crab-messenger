#[derive(Clone, Copy)]
pub enum TabState {
    Login,
    Home,
}

impl Default for TabState {
    fn default() -> Self {
        TabState::Login
    }
}
