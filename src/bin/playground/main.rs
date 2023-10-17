use crab_messenger::auth::{Auth, AuthModuleImpl};
use crab_messenger::RootModule;
use shaku::HasComponent;
use std::sync::Arc;

use crab_messenger::client::run_app;

fn main() -> anyhow::Result<()> {
    run_app();
    Ok(())
}
