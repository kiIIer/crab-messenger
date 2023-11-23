-- Your SQL goes here

CREATE TABLE users_chats
(
    user_id text not null references users (id),
    chat_id int not null references chats(id),
    primary key (user_id, chat_id)
);