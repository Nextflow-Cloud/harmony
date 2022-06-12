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
use warp::Filter;

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

    socket::start_server().await;
    println!("Server is running on port 9000");

    // leaving space for background tasks

    let rpc = warp::post()
        .and(warp::path("api/rpc"))
        .and(warp::body::bytes())
        .map(methods::rpc::routes);

    warp::serve(rpc).run(([0, 0, 0, 0], 8080)).await;
}
