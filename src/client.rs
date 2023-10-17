use crate::client::input::Input;
use crate::client::redux::actions::Action;
use crate::client::redux::Store;
use crate::client::view::AppComponent;
use anyhow::bail;
use crossbeam_channel::{Receiver, Select};
use crossterm::event::Event;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::error::Error;
use std::io;
use std::io::Write;

mod input;
mod redux;
mod view;

pub fn run_app() -> anyhow::Result<()> {
    setup_terminal();

    let mut terminal = start_terminal(io::stdout())?;
    let input: Input = Input::new();
    let store: Store = Store::default();
    let app_component: AppComponent = AppComponent::default();

    loop {
        let action = select_action(&input.receiver)?;
        store.dispatch(action);
        let state = store.get_state();
        terminal.draw(|f| app_component.draw(f, state))?;
    }
    shutdown_terminal();
}

fn select_action(rx_input: &Receiver<Event>) -> anyhow::Result<Action> {
    let mut sel = Select::new();
    sel.recv(rx_input);
    let oper = sel.select();
    let index = oper.index();

    let action = match index {
        0 => oper.recv(rx_input).map(Action::Input),
        _ => bail!("unknown select source"),
    }?;

    Ok(action)
}

fn setup_terminal() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn shutdown_terminal() {
    io::stdout().execute(LeaveAlternateScreen);

    disable_raw_mode();
}

fn start_terminal<W: Write>(buf: W) -> io::Result<Terminal<CrosstermBackend<W>>> {
    let backend = CrosstermBackend::new(buf);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(terminal)
}
