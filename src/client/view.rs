use crate::client::redux::state::State;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::Rect;
use ratatui::Frame;
use shaku::Interface;
use std::io;

pub mod app;

pub trait View {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()>;
}
