use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::server::persistence::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub email: String,
}
