use diesel::prelude::*;
use prost_types::Timestamp;
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

impl From<Invite> for crate::utils::messenger::Invite {
    fn from(invite: Invite) -> Self {
        Self {
            id: invite.id,
            inviter_user_id: invite.inviter_user_id,
            invitee_user_id: invite.invitee_user_id,
            chat_id: invite.chat_id,
            created_at: Some(Timestamp {
                seconds: invite.created_at.timestamp(),
                nanos: invite.created_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

impl From<crate::utils::messenger::Invite> for Invite {
    fn from(value: crate::utils::messenger::Invite) -> Self {
        let timestamp = value.created_at.unwrap();

        Invite {
            id: value.id,
            inviter_user_id: value.inviter_user_id,
            invitee_user_id: value.invitee_user_id,
            chat_id: value.chat_id,
            created_at: chrono::NaiveDateTime::from_timestamp_opt(
                timestamp.seconds,
                timestamp.nanos as u32,
            )
            .unwrap(),
        }
    }
}
