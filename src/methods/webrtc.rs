use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::services::database::members::Member;
use crate::services::database::spaces::Space;
use crate::services::permissions::Permission;
use crate::services::socket::RpcClient;
use crate::services::webrtc::ActiveCall;

use super::{Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCallMethod {
    id: String,
    space_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCallResponse {
    token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RtcAuthorization {
    channel_id: String,
    user_id: String,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for JoinCallMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?; // TODO: check rate limit, permissions req'd
        if let Some(space_id) = &self.space_id {
            let space = Space::get(&space_id).await?;
            if !space.members.contains(&id) {
                return Err(Error::NotFound); // unauthorized
            }
            let member = Member::get(&id, &space.id).await?;
            let channel = space.get_channel(&self.id).await?;
            let permission = member
                .get_permission_in_channel(&channel, Permission::JoinCalls)
                .await?;
            if !permission {
                return Err(Error::MissingPermission {
                    permission: Permission::JoinCalls,
                });
            }
            let call = ActiveCall::get_in_channel(space_id, &self.id).await?;
            if let Some(mut call) = call {
                call.join_user(id.clone()).await?;
                let token = call.get_token(&id).await?;
                Ok(Response::JoinCall(JoinCallResponse { token }))
            } else {
                Err(Error::NotFound)
            }
        } else {
            Err(Error::Unimplemented)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StartCallMethod {
    id: String,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for StartCallMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        if let Some(space_id) = &self.space_id {
            let space = Space::get(&space_id).await?;
            if !space.members.contains(&id) {
                return Err(Error::NotFound);
            }
            let member = Member::get(&id, &space.id).await?;
            let channel = space.get_channel(&self.id).await?;
            let permission = member
                .get_permission_in_channel(&channel, Permission::StartCalls)
                .await?;
            if !permission {
                return Err(Error::MissingPermission {
                    permission: Permission::StartCalls,
                });
            }
            let call = ActiveCall::create(space_id, &self.id, &id).await?;
            let token = call.get_token(&id).await?;
            Ok(Response::StartCall(StartCallResponse { token }))
        } else {
            Err(Error::Unimplemented)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StartCallResponse {
    token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndCallMethod {
    id: String,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for EndCallMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        if let Some(space_id) = &self.space_id {
            let space = Space::get(&space_id).await?;
            if !space.members.contains(&id) {
                return Err(Error::NotFound);
            }
            let member = Member::get(&id, &space.id).await?;
            let channel = space.get_channel(&self.id).await?;
            let permission = member
                .get_permission_in_channel(&channel, Permission::ManageCalls)
                .await?;
            if !permission {
                return Err(Error::MissingPermission {
                    permission: Permission::ManageCalls,
                });
            }
            let call = ActiveCall::get_in_channel(space_id, &self.id).await?;
            if let Some(call) = call {
                call.end().await?;
                Ok(Response::EndCall(EndCallResponse {}))
            } else {
                Err(Error::NotFound)
            }
        } else {
            Err(Error::Unimplemented)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndCallResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaveCallMethod {
    id: String,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for LeaveCallMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        if let Some(space_id) = &self.space_id {
            let call = ActiveCall::get_in_channel(space_id, &self.id).await?;
            if let Some(mut call) = call {
                if call.members.contains(&id) {
                    return Err(Error::NotFound);
                }
                call.leave_user(&id.clone()).await?;
                Ok(Response::LeaveCall(LeaveCallResponse {}))
            } else {
                Err(Error::NotFound)
            }
        } else {
            Err(Error::Unimplemented)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaveCallResponse {}
