use rand::distributions::Alphanumeric;
use rand::Rng;

pub mod auth;
pub mod messenger;
pub mod rabbit_channel_manager;

pub mod db_connection_manager;
pub mod persistence;

pub mod rabbit_declares;

pub mod rabbit_types;

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
