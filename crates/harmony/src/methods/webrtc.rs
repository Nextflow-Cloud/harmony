use std::sync::Arc;

use dashmap::DashMap;
use rapid::socket::{RpcClient, RpcResponder, RpcValue};
use serde::{Deserialize, Serialize};

use crate::authentication::check_authenticated;
use crate::errors::{Error, Result};
use crate::services::database::members::Member;
use crate::services::database::spaces::Space;
use crate::services::permissions::Permission;
use crate::services::webrtc::ActiveCall;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCallMethod {
    id: String,
    space_id: Option<String>,
    sdp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCallResponse {
    sdp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RtcAuthorization {
    channel_id: String,
    user_id: String,
    space_id: Option<String>,
}

async fn join_call(
    clients: Arc<DashMap<String, RpcClient>>,
    id: String,
    data: JoinCallMethod,
) -> impl RpcResponder {
    check_authenticated(clients, &id)?; // TODO: check rate limit, permissions req'd
    if let Some(_space_id) = &data.space_id {
        // let space = Space::get(space_id).await?;
        // if !space.members.contains(&id) {
        //     return Err(Error::NotFound); // unauthorized
        // }
        // let member = Member::get(&id, &space.id).await?;
        // let channel = space.get_channel(&self.id).await?;
        // let permission = member
        //     .get_permission_in_channel(&channel, Permission::JoinCalls)
        //     .await?;
        // if !permission {
        //     return Err(Error::MissingPermission {
        //         permission: Permission::JoinCalls,
        //     });
        // }
        // let call = ActiveCall::get_in_channel(space_id, &self.id).await?;
        // if let Some(mut call) = call {
        //     call.join_user(id.clone()).await?;
        //     let token = call.get_token(&id).await?;
        //     Ok(Response::JoinCall(JoinCallResponse { token }))
        // } else {
        //     Err(Error::NotFound)
        // }
        Err::<RpcValue<JoinCallResponse>, _>(Error::NoVoiceNodesAvailable)
    } else {
        Err(Error::Unimplemented)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StartCallMethod {
    id: String,
    space_id: Option<String>,
}

async fn start_call(
    clients: Arc<DashMap<String, RpcClient>>,
    id: String,
    data: StartCallMethod,
) -> impl RpcResponder {
    check_authenticated(clients, &id)?;
    if let Some(space_id) = &data.space_id {
        let space = Space::get(space_id).await?;
        if !space.members.contains(&id) {
            return Err(Error::NotFound);
        }
        let member = Member::get(&id, &space.id).await?;
        let channel = space.get_channel(&data.id).await?;
        let permission = member
            .get_permission_in_channel(&channel, Permission::StartCalls)
            .await?;
        if !permission {
            return Err(Error::MissingPermission {
                permission: Permission::StartCalls,
            });
        }
        let call = ActiveCall::create(space_id, &data.id, &id).await?;
        let token = call.get_token(&id).await?;
        Ok(RpcValue(StartCallResponse { token }))
    } else {
        Err(Error::Unimplemented)
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

async fn end_call(
    clients: Arc<DashMap<String, RpcClient>>,
    id: String,
    data: EndCallMethod,
) -> impl RpcResponder {
    check_authenticated(clients, &id)?;
    if let Some(space_id) = &data.space_id {
        let space = Space::get(space_id).await?;
        if !space.members.contains(&id) {
            return Err(Error::NotFound);
        }
        let member = Member::get(&id, &space.id).await?;
        let channel = space.get_channel(&data.id).await?;
        let permission = member
            .get_permission_in_channel(&channel, Permission::ManageCalls)
            .await?;
        if !permission {
            return Err(Error::MissingPermission {
                permission: Permission::ManageCalls,
            });
        }
        let call = ActiveCall::get_in_channel(space_id, &data.id).await?;
        if let Some(call) = call {
            call.end().await?;
            Ok(RpcValue(EndCallResponse {}))
        } else {
            Err(Error::NotFound)
        }
    } else {
        Err(Error::Unimplemented)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndCallResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaveCallMethod {
    id: String,
    space_id: Option<String>,
}

async fn leave_call(
    clients: Arc<DashMap<String, RpcClient>>,
    id: String,
    data: LeaveCallMethod,
) -> impl RpcResponder {
    check_authenticated(clients, &id)?;
    if let Some(space_id) = &data.space_id {
        let call = ActiveCall::get_in_channel(space_id, &data.id).await?;
        if let Some(mut call) = call {
            if call.members.contains(&id) {
                return Err(Error::NotFound);
            }
            call.leave_user(&id.clone()).await?;
            Ok(RpcValue(LeaveCallResponse {}))
        } else {
            Err(Error::NotFound)
        }
    } else {
        Err(Error::Unimplemented)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaveCallResponse {}
