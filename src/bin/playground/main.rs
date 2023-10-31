use crab_messenger::client::{build_client_module, Client};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use crossterm::{
    execute,
    terminal::{size, ScrollUp, SetSize},
};
use shaku::HasComponent;
use std::io;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let module = build_client_module();
    let client: Arc<dyn Client> = module.resolve();
    client.run_client()?;
    Ok(())
}
