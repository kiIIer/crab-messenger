use crate::auth::AuthModule;
use crate::auth::{build_auth_module, Auth};
use crate::client::{build_client_module, ClientModule};
use shaku::module;
use std::sync::Arc;

pub mod auth;
pub mod client;
pub mod messenger;
pub mod server;

module! {
    pub RootModule{
        components = [],
        providers = [],
        use AuthModule{
            components = [dyn Auth],
            providers = []
        },
        use ClientModule{
            components = [],
            providers = [],
        }
    }
}

fn build_root_module() -> Arc<RootModule> {
    Arc::new(RootModule::builder(build_auth_module(), build_client_module()).build())
}
