use anyhow::Result;
use crab_messenger::worker::{build_worker_module, Worker};
use shaku::HasComponent;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let module = build_worker_module();
    let server: Arc<dyn Worker> = module.resolve();
    server.run_worker().await?;
    Ok(())
}
