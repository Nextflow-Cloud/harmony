#![feature(arbitrary_enum_discriminant)]
#![feature(async_closure)]
#![feature(slice_pattern)]
#![allow(dead_code)]

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
    // TODO: logger, environment
    // TODO: negotiate encryption

    database::connect().await;
    println!("Database is connected");
    // run DB migrations as necessary

    webrtc::create_workers().await;
    println!("SFU workers have spawned");

    // leaving space for background tasks

    println!("Listening on port 9000");
    socket::start_server().await;
}
