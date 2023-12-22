use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::utils::persistence::schema::users_chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Book))]
#[diesel(belongs_to(Author))]
#[diesel(primary_key(user_id, chat_id))]
pub struct UsersChats {
    pub user_id: String,
    pub chat_id: i32,
}
