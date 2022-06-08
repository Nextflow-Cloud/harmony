use std::sync::Arc;

use async_std::net::TcpStream;
use async_std::sync::Mutex;
use async_tungstenite::WebSocketStream;

use crate::methods::{IdentifyMethod, Response, IdentifyResponse};

pub fn identify(socket: Arc<Mutex<WebSocketStream<TcpStream>>>, method: IdentifyMethod) -> Response {

    println!("Public key: {:?}", method.public_key);
    println!("Token: {:?}", method.token);
    Response::Identify(IdentifyResponse {
        success: true
    })
}
