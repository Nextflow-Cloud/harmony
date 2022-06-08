use std::{env, net::Ipv4Addr};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MONGODB_URI: String = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    pub static ref MONGODB_DATABASE: String =
        env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");
    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    pub static ref LISTEN_ADDRESS: String =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    pub static ref PUBLIC_ADDRESS: Ipv4Addr = env::var("PUBLIC_ADDRESS")
        .expect("PUBLIC_ADDRESS must be set")
        .parse::<Ipv4Addr>()
        .unwrap();
    pub static ref PUBLIC_URL: String = env::var("PUBLIC_URL").expect("PUBLIC_URL must be set");
}
