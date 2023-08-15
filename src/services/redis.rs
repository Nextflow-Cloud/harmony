use once_cell::sync::OnceCell;
use redis::{Client, aio::Connection};

use super::environment::REDIS_URI;

static REDIS: OnceCell<Client> = OnceCell::new();

pub async fn connect() {
    let client = Client::open(&**REDIS_URI).expect("Failed to connect");
    REDIS.set(client).expect("Failed to set client");
}

pub fn get_client() -> &'static Client {
    REDIS.get().expect("Failed to get client")
}

pub async fn get_connection() -> Connection {
    let connection = get_client().get_async_std_connection().await.expect("Failed to get connection");
    connection
}
