use crate::utils::messenger::Message as ProtoMessage;
use diesel::{Insertable, Queryable, Selectable};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::server::persistence::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: i32,
    pub text: String,
    pub created_at: chrono::NaiveDateTime,
    pub user_id: String,
    pub chat_id: i32,
}

#[derive(Queryable, Selectable, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::server::persistence::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertMessage {
    pub text: String,
    pub user_id: String,
    pub chat_id: i32,
}

impl From<ProtoMessage> for Message {
    fn from(proto_msg: ProtoMessage) -> Self {
        let timestamp = proto_msg.created_at.unwrap(); // Safe to unwrap based on your constraints

        Message {
            id: proto_msg.id,
            user_id: proto_msg.user_id,
            chat_id: proto_msg.chat_id,
            text: proto_msg.text,
            created_at: chrono::NaiveDateTime::from_timestamp_opt(
                timestamp.seconds,
                timestamp.nanos as u32,
            )
            .unwrap(),
        }
    }
}

impl From<Message> for ProtoMessage {
    fn from(diesel_msg: Message) -> Self {
        // You would typically ensure this conversion cannot fail by making sure the fields match up correctly.
        // If created_at is a NaiveDateTime, you must convert it to a Timestamp.
        let created_at = Timestamp {
            // Convert the NaiveDateTime to seconds and nanoseconds
            seconds: diesel_msg.created_at.timestamp(),
            nanos: diesel_msg.created_at.timestamp_subsec_nanos() as i32,
        };

        ProtoMessage {
            id: diesel_msg.id,
            user_id: diesel_msg.user_id,
            chat_id: diesel_msg.chat_id,
            text: diesel_msg.text,
            created_at: Some(created_at), // gRPC Timestamp is typically wrapped in an Option
        }
    }
}
