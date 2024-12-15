#![allow(dead_code)]

pub mod errors;
pub mod methods;
pub mod services;
pub mod authentication;

use authentication::authenticate;
use rapid::socket::RpcServer;
use services::database;
use services::redis;
// use services::webrtc;

use log::info;
use services::webrtc;

use crate::services::environment::LISTEN_ADDRESS;

#[async_std::main]
async fn main() {
    // TODO: environment, negotiate encryption

    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    database::connect().await;
    info!("Connected to database");

    // run DB migrations as necessary

    redis::connect().await;
    info!("Connected to Redis");
    webrtc::spawn_check_available_nodes();

    let listen_address = LISTEN_ADDRESS.to_owned();
    info!("Starting server at {listen_address}");
    let server = RpcServer::new(Box::new(|token| Box::pin(authenticate(token))));
    server.start(listen_address).await;
}
