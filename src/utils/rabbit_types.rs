use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RabbitInviteAccept {
    pub invite_id: i32,
    pub user_id: String,
}
