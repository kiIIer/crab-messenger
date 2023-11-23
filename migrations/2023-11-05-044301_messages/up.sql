-- Your SQL goes here

create table messages
(
    id         serial primary key,
    text       text        not null,
    created_at timestamptz not null default (now() at time zone 'utc'),
    user_id    text        not null references public.users (id),
    chat_id    int         not null references public.chats (id)
)