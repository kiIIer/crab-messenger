use crate::auth::Auth;
use crate::auth::AuthModule;
use shaku::module;

pub mod auth;
pub mod messenger;
pub mod client;

module! {
    pub RootModule{
    components =[],
    providers=[],
        use dyn AuthModule{
            components =[dyn Auth],
            providers = []
        }
    }
}
