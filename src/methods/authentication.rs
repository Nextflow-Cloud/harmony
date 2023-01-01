use std::collections::HashSet;

use async_trait::async_trait;
use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::Error;
use crate::methods::{ErrorResponse, Response};
use crate::services::encryption::{generate, random_number};
use crate::services::environment::JWT_SECRET;
use crate::services::socket::RpcClient;

use super::Respond;

#[derive(Deserialize)]
struct User {
    // TODO: Find the other properties
    id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyMethod {
    pub public_key: Vec<u8>,
    pub token: String,
}

// Important: This only accepts a token and will not sign a token.
// The token is to be obtained from a separate login server
// (e.g. SSO system)
#[async_trait]
impl Respond for IdentifyMethod {
    async fn respond(
        &self,
        clients: DashMap<String, RpcClient>,
        id: String,
    ) -> Response {
        println!("Public key: {:?}", self.public_key);
        println!("Token: {:?}", self.token);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::new();
        validation.validate_exp = false;
        let token_message = decode::<User>(
            &self.token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &validation,
        );
        match token_message {
            Ok(r) => {
                let mut client = clients.get_mut(&id).unwrap();
                client.user_id = Some(r.claims.id);
                Response::Identify(IdentifyResponse { success: true })
            }
            Err(_) => Response::Error(ErrorResponse {
                error: Error::InvalidToken,
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyResponse {
    pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMethod {}

#[async_trait]
impl Respond for HeartbeatMethod {
    async fn respond(
        &self,
        clients: DashMap<String, RpcClient>,
        id: String,
    ) -> Response {
        let client = clients.get(&id).unwrap();
        let tx = client.heartbeat_tx.lock().await;
        tx.send(()).unwrap();
        Response::Heartbeat(HeartbeatResponse {})
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIdMethod {}

#[async_trait]
impl Respond for GetIdMethod {
    async fn respond(
        &self,
        clients: DashMap<String, RpcClient>,
        id: String,
    ) -> Response {
        let client = clients.get(&id).unwrap();
        let mut request_ids = client.request_ids.lock().await;
        let mut new_request_ids = Vec::new();
        for _ in 0..20 {
            let id = generate(
                random_number,
                &[
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                ],
                10,
            );
            request_ids.push(id.clone());
            new_request_ids.push(id);
        }
        Response::GetId(GetIdResponse {
            request_ids: new_request_ids,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIdResponse {
    pub request_ids: Vec<String>,
}
