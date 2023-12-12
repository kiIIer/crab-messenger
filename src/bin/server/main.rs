use crab_messenger::server::{build_server_module, Server};
use shaku::HasComponent;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let module = build_server_module();
    let server: Arc<dyn Server> = module.resolve();
    server.run_server().await?;
    Ok(())
}
