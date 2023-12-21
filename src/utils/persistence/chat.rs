use crate::utils::messenger::Chat as ProtoChat;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Queryable, Selectable, Deserialize, Insertable)]
#[diesel(table_name = crate::utils::persistence::schema::chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chat {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Deserialize, Insertable, Debug)]
#[diesel(table_name = crate::utils::persistence::schema::chats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertChat {
    pub name: String,
}

impl From<Chat> for ProtoChat {
    fn from(diesel_chat: Chat) -> Self {
        ProtoChat {
            id: diesel_chat.id,
            name: diesel_chat.name,
        }
    }
}
