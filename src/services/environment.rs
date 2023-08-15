use std::env;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MONGODB_URI: String = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    pub static ref MONGODB_DATABASE: String =
        env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set");
    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    pub static ref MAX_SPACE_COUNT: i16 = env::var("MAX_SPACE_COUNT")
        .unwrap_or_else(|_| "200".to_string())
        .parse::<i16>()
        .expect("MAX_SPACE_COUNT must be an integer");
    pub static ref LISTEN_ADDRESS: String =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:9000".to_string());
    pub static ref REDIS_URI: String = env::var("REDIS_URI").expect("REDIS_URI must be set");
}
