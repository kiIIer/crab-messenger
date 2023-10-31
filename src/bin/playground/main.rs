use crab_messenger::client::{build_client_module, Client};
use shaku::HasComponent;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let module = build_client_module();
    let client: Arc<dyn Client> = module.resolve();
    client.run_client()?;
    Ok(())
}
