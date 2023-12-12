use std::sync::Arc;

use shaku::HasComponent;
use tracing_subscriber::{fmt, EnvFilter};

use crab_messenger::client::{build_client_module, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let module = build_client_module();
    let client: Arc<dyn Client> = module.resolve();
    client.run_client()?;
    Ok(())
}
