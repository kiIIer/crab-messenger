use crate::utils::auth::{AuthState, StartFlowResponse};
use crate::client::redux::state::State;
use crossterm::event::Event;

#[derive(Clone)]
pub enum Action {
    Input(Event),
    StartLogin,
    Login(StartFlowResponse),
    LoginSuccess(AuthState),
    Tick,
}

pub enum ReduceResult {
    Consumed(State),
    ConsumedButKindaNot,
    Ignored,
}
