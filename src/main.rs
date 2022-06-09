#![feature(arbitrary_enum_discriminant)]
#![feature(async_closure)]

pub mod globals;
pub mod methods;
pub mod services;

use std::thread::sleep;
use std::time::Duration;

use services::database;
use services::socket;
use services::webrtc;

// use std::io::Write;

// use log::info;

#[async_std::main]
async fn main() {
    // TODO: logger, environment

    database::connect().await;
    println!("Database is connected");
    // run DB migrations as necessary

    webrtc::create_workers().await;
    println!("SFU workers have spawned");

    socket::start_server().await;
    println!("Server is running on port 9000");

    // leaving space for background tasks

    loop {
        sleep(Duration::from_secs(1))
    }
}
