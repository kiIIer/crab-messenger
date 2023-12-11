use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::utils::persistence::schema::chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chat {
    pub id: i32,
    pub name: String,
}
