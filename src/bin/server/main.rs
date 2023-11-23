use crab_messenger::server::{build_server_module, Server};
use shaku::HasComponent;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let module = build_server_module();
    let server: Arc<dyn Server> = module.resolve();
    server.run_server().await?;
    Ok(())
}
