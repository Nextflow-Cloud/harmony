pub mod channels;
pub mod scopes;
pub mod users;
pub mod messages;

use crate::services::environment::{MONGODB_DATABASE, MONGODB_URI};

use mongodb::{Client, Database};
use once_cell::sync::OnceCell;

static DATABASE: OnceCell<Client> = OnceCell::new();

pub async fn connect() {
    let client = Client::with_uri_str(&*MONGODB_URI)
        .await
        .expect("Failed to connect to MongoDB");
    DATABASE.set(client).expect("Failed to set MongoDB client");
}

pub fn get_connection() -> &'static Client {
    DATABASE.get().expect("Failed to get MongoDB client")
}

pub fn get_database() -> Database {
    get_connection().database(&MONGODB_DATABASE)
}
