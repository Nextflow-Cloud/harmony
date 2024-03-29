use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::methods::Response;
use crate::services::database::users::User;
use crate::services::encryption::generate_id;
use crate::services::environment::JWT_SECRET;
use crate::services::socket::RpcClient;

use super::Respond;

#[derive(Deserialize)]
struct UserJwt {
    // TODO: Find the other properties
    id: String,
    issued_at: u128,
    expires_at: u128,
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
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        println!("Public key: {:?}", self.public_key);
        println!("Token: {:?}", self.token);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::new();
        validation.validate_exp = false;
        let token_message = decode::<UserJwt>(
            &self.token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &validation,
        )
        .map_err(|_| Error::InvalidToken)?;
        let time = chrono::Utc::now().timestamp_millis() as u128;
        if time > token_message.claims.expires_at {
            return Err(Error::InvalidToken);
        }
        let mut client = clients.get_mut(&id).unwrap();
        let user = User::get(&token_message.claims.id).await;
        let user = if let Err(Error::NotFound) = user {
            User::create(token_message.claims.id).await?
        } else {
            user?
        };
        client.user = Some(Arc::new(user));
        Ok(Response::Identify(IdentifyResponse { success: true }))
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
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        let client = clients.get(&id).unwrap();
        client.heartbeat_tx.send(()).await.unwrap();
        Ok(Response::Heartbeat(HeartbeatResponse { ack: true }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatResponse {
    ack: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIdMethod {}

#[async_trait]
impl Respond for GetIdMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        let mut client = clients.get_mut(&id).unwrap();
        let mut new_request_ids = Vec::new();
        for _ in 0..20 {
            let id = generate_id();
            client.request_ids.push(id.clone());
            new_request_ids.push(id);
        }
        Ok(Response::GetId(GetIdResponse {
            request_ids: new_request_ids,
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIdResponse {
    pub request_ids: Vec<String>,
}

pub fn check_authenticated(
    clients: Arc<DashMap<String, RpcClient>>,
    id: &str,
) -> Result<Arc<User>> {
    let client = clients.get(id).expect("Failed to get client");
    if let Some(x) = &client.user {
        Ok(x.clone())
    } else {
        Err(Error::NotAuthenticated)
    }
}
