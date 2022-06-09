use std::sync::Arc;

use async_std::net::TcpStream;
use async_std::sync::Mutex;
use async_tungstenite::WebSocketStream;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::methods::{IdentifyMethod, IdentifyResponse, NotFoundResponse, Response};
use crate::services::environment::JWT_SECRET;

#[derive(Deserialize)]
struct User {
    // TODO: Find the other properties
    email: String,
}

// Important: This only accepts a token and will not sign a token.
// The token is to be obtained from a separate login server
// (e.g. SSO system)
pub fn identify(
    socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    method: IdentifyMethod,
) -> Response {
    println!("Public key: {:?}", method.public_key);
    println!("Token: {:?}", method.token);
    let token_message = decode::<User>(
        &method.token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::new(Algorithm::HS256),
    );
    match token_message {
        Ok(r) => Response::Identify(IdentifyResponse { success: true }),
        Err(e) => Response::NotFound(NotFoundResponse {
            error: "Invalid token".to_string(),
        }),
    }
}
