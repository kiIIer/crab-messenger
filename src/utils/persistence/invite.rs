use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::utils::persistence::schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invite {
    pub id: i32,
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub chat_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Queryable)]
#[diesel(table_name = crate::utils::persistence::schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertInvite {
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub chat_id: i32,
}
