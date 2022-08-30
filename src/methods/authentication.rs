use std::collections::HashSet;
use std::sync::Arc;

use async_std::sync::Mutex;

use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::methods::{ErrorResponse, IdentifyMethod, IdentifyResponse, Response};
use crate::services::environment::JWT_SECRET;
use crate::services::socket::VoiceClient;

#[derive(Deserialize)]
struct User {
    // TODO: Find the other properties
    id: String,
}

// Important: This only accepts a token and will not sign a token.
// The token is to be obtained from a separate login server
// (e.g. SSO system)
pub async fn identify(
    method: IdentifyMethod,
    clients: Arc<Mutex<DashMap<String, VoiceClient>>>,
    id: String,
) -> Response {
    println!("Public key: {:?}", method.public_key);
    println!("Token: {:?}", method.token);
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;
    let token_message = decode::<User>(
        &method.token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &validation,
    );
    match token_message {
        Ok(r) => {
            let clients_locked = clients.lock().await;
            let mut client = clients_locked.get_mut(&id).unwrap();
            client.user_id = Some(r.claims.id);
            Response::Identify(IdentifyResponse { success: true })
        }
        Err(_) => Response::Error(ErrorResponse {
            error: "Invalid token".to_string(),
        }),
    }
}
