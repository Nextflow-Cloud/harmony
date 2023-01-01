use lazy_static::lazy_static;

lazy_static! {
    pub static ref HEARTBEAT_INTERVAL: u64 = 30000;
    pub static ref HEARTBEAT_TIMEOUT: u64 = 60000;
}
