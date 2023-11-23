-- Your SQL goes here
CREATE TABLE invites
(
    id              serial primary key,
    inviter_user_id text        not null references users (id),
    invitee_user_id text        not null references users (id),
    chat_id         int         not null references chats (id),
    created_at      timestamptz not null default (now() at time zone 'utc')
);