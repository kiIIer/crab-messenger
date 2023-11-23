use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::server::persistence::schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invite {
    pub id: i32,
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub chat_id: i32,
    pub created_at: chrono::NaiveDateTime,
}
