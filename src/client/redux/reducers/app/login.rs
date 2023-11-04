use crate::auth::AuthModule;
use crate::auth::{build_auth_module, Auth};
use crate::client::redux::action::Action::LoginSuccess;
use crate::client::redux::action::{Action, ReduceResult};
use crate::client::redux::reducers::Reducer;
use crate::client::redux::state::State;
use crossbeam_channel::Sender;
use shaku::{module, Component, Interface};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;

pub trait LoginReducer: Reducer + Interface {}
#[derive(Component)]
#[shaku(interface = LoginReducer)]
pub struct LoginReducerImpl {
    #[shaku(inject)]
    auth: Arc<dyn Auth>,
}

impl Reducer for LoginReducerImpl {
    fn reduce(
        &self,
        action: &Action,
        state: &State,
        dispatch_tx: Sender<Action>,
        handle: Handle,
    ) -> ReduceResult {
        match action {
            Action::StartLogin => {
                let tx = dispatch_tx.clone();
                let auth = self.auth.clone();
                handle.spawn(async move {
                    let result = auth.start_device_flow().await;
                    if let Ok(response) = result {
                        tx.send(Action::Login(response))
                            .expect("Couldn't send the stuff");
                    }
                });
                ReduceResult::ConsumedButKindaNot
            }
            Action::Login(params) => {
                let tx = dispatch_tx.clone();
                let auth = self.auth.clone();
                let mut new_state = state.clone();
                new_state.code = Some(params.user_code.clone());
                new_state.link = Some(params.verification_uri.clone());

                let my_params = params.clone();
                handle.spawn(async move {
                    let result = auth
                        .poll_access_token(&my_params.device_code, my_params.interval)
                        .await;
                    if let Ok(poll_response) = result {
                        tx.send(Action::LoginSuccess(poll_response))
                            .expect("Couldn't send the stuff");
                    }
                });

                let link = params.verification_uri.clone();
                let code = params.user_code.clone();
                handle.spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    open::that(format!("{}?user_code={}", link, code))
                });
                ReduceResult::Consumed(new_state)
            }
            Action::LoginSuccess(auth_state) => {
                let mut new_state = state.clone();
                let refresh_token = auth_state.refresh_token.clone();
                let expires_in = auth_state.expires_in;
                new_state.auth_state = Some(auth_state.clone());

                let tx = dispatch_tx.clone();
                let auth = self.auth.clone();

                handle.spawn(async move {
                    tokio::time::sleep(Duration::from_secs(expires_in as u64)).await;
                    let result = auth.request_refresh_token(&refresh_token).await;

                    if let Ok(new_auth) = result {
                        tx.send(LoginSuccess(new_auth))
                            .expect("Couldn't send the stuff");
                    }
                });

                ReduceResult::Consumed(new_state)
            }
            _ => ReduceResult::Ignored,
        }
    }
}

impl LoginReducer for LoginReducerImpl {}

module! {
    pub ReducersLoginModule {
        components = [LoginReducerImpl],
        providers = [],
        use AuthModule{
            components = [dyn Auth],
            providers = [],
        }
    }
}

pub fn build_reducers_login_module() -> Arc<ReducersLoginModule> {
    Arc::new(ReducersLoginModule::builder(build_auth_module()).build())
}
