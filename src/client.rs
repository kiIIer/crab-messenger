use crate::client::input::{build_input_module, Input, InputModule};
use crate::client::redux::action::Action;
use crate::client::redux::state::State;
use crate::client::redux::store::build_store_module;
use crate::client::redux::{build_redux_module, store::Store, store::StoreModule};
use crate::client::view::app::{build_app_view_module, AppView, AppViewModule};
use crossbeam_channel::Sender;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scopeguard::defer;
use shaku::{module, Component, Interface};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use std::{io, panic, thread};
use tokio::runtime::Handle;

mod input;
mod redux;
mod view;
pub trait Client: Interface {
    fn run_client(self: Arc<Self>) -> anyhow::Result<()>;
}

#[derive(Component)]
#[shaku(interface = Client)]
pub struct ClientImpl {
    #[shaku(inject)]
    store: Arc<dyn Store>,

    #[shaku(inject)]
    view: Arc<dyn AppView>,

    #[shaku(inject)]
    input: Arc<dyn Input>,
}

impl Client for ClientImpl {
    fn run_client(self: Arc<Self>) -> anyhow::Result<()> {
        self.setup_terminal()?;

        defer! {
            self.shutdown_terminal();
        }

        self.clone().set_panic_handlers()?;

        self.store.get_dispatch().send(Action::StartLogin).unwrap();

        let select = self.store.get_select();

        let mut terminal = self.start_terminal(io::stdout())?;

        self.store.process(Handle::current()).unwrap();
        self.input.clone().process(self.store.clone());

        let tick_dispatch = self.store.get_dispatch();
        tokio::spawn(async move {
            loop {
                tick_dispatch.send(Action::Tick).unwrap();

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        while let Ok(state) = select.recv() {
            if state.should_exit {
                break;
            }
            terminal.draw(|f| {
                self.view.draw(f, f.size(), state).expect("Couldn't draw");
            })?;
        }

        Ok(())
    }
}
impl ClientImpl {
    fn start_terminal<W: Write>(&self, buf: W) -> io::Result<Terminal<CrosstermBackend<W>>> {
        let backend = CrosstermBackend::new(buf);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        Ok(terminal)
    }

    fn setup_terminal(&self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    fn shutdown_terminal(&self) {
        let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

        if let Err(e) = leave_screen {
            eprintln!("leave_screen failed:\n{e}");
        }

        let leave_raw_mode = disable_raw_mode();

        if let Err(e) = leave_raw_mode {
            eprintln!("leave_raw_mode failed:\n{e}");
        }
    }

    fn set_panic_handlers(self: Arc<Self>) -> anyhow::Result<()> {
        let self1 = self.clone();
        ctrlc::set_handler(move || {
            self1.shutdown_terminal();
            std::process::exit(1);
        })
        .expect("Error setting Ctrl-C handler");

        let default_panic_hook = panic::take_hook();

        panic::set_hook(Box::new(move |e| {
            self.shutdown_terminal();

            default_panic_hook(e);
        }));

        Ok(())
    }
}

module! {
    pub ClientModule {
        components = [ClientImpl],
        providers = [],
        use StoreModule{
            components = [dyn Store],
            providers = [],
        },
        use AppViewModule{
            components = [dyn AppView],
            providers = [],
        },
        use InputModule{
            components = [dyn Input],
            providers = [],
        }
    }
}

pub fn build_client_module() -> Arc<ClientModule> {
    Arc::new(
        ClientModule::builder(
            build_store_module(),
            build_app_view_module(),
            build_input_module(),
        )
        .build(),
    )
}
