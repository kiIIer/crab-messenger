// @generated automatically by Diesel CLI.

diesel::table! {
    chats (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    invites (id) {
        id -> Int4,
        inviter_user_id -> Text,
        invitee_user_id -> Text,
        chat_id -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        id -> Int4,
        text -> Text,
        created_at -> Timestamptz,
        user_id -> Text,
        chat_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
    }
}

diesel::table! {
    users_chats (user_id, chat_id) {
        user_id -> Text,
        chat_id -> Int4,
    }
}

diesel::joinable!(invites -> chats (chat_id));
diesel::joinable!(messages -> chats (chat_id));
diesel::joinable!(messages -> users (user_id));
diesel::joinable!(users_chats -> chats (chat_id));
diesel::joinable!(users_chats -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    chats,
    invites,
    messages,
    users,
    users_chats,
);
