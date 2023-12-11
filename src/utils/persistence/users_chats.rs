use diesel::{Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::utils::persistence::schema::users_chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UsersChats {
    pub user_id: String,
    pub chat_id: i32,
}
