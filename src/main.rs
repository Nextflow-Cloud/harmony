#![feature(arbitrary_enum_discriminant)]

pub mod globals;
pub mod methods;
pub mod services;

use services::database;
use services::socket;
use services::webrtc;

// use std::io::Write;

// use log::info;

#[async_std::main]
async fn main() {
    database::connect().await;
    println!("Database is connected");
    webrtc::create_workers().await;
    println!("SFU workers have spawned");
    socket::start_server().await;
    println!("Server is running on port 9000");
}
