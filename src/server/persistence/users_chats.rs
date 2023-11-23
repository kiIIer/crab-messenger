use diesel::{Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::server::persistence::schema::users_chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UsersChats {
    pub user_id: String,
    pub chat_id: i32,
}
