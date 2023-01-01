#![allow(dead_code)]

pub mod globals;
pub mod methods;
pub mod services;
pub mod errors;

use services::database;
use services::socket;
// use services::webrtc;

use log::info;

use crate::services::environment::LISTEN_ADDRESS;

#[async_std::main]
async fn main() {
    // TODO: environment, negotiate encryption

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    database::connect().await;
    info!("Connected to database");

    // run DB migrations as necessary
    
    // webrtc::create_workers().await;
    // println!("SFU workers have spawned");

    let listen_address = LISTEN_ADDRESS.to_owned();
    info!("Starting server at {listen_address}");
    socket::start_server().await;
}
