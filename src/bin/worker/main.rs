use anyhow::Result;
use crab_messenger::worker::{build_worker_module, Worker};
use std::sync::Arc;
use shaku::HasComponent;

#[tokio::main]
async fn main() -> Result<()> {
    let module = build_worker_module();
    let server: Arc<dyn Worker> = module.resolve();
    server.run_worker().await?;
    Ok(())
}
