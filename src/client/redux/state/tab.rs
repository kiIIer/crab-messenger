#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub enum TabState {
    Login,
    Home,
    Chats,
    Users,
}

impl Default for TabState {
    fn default() -> Self {
        TabState::Login
    }
}
