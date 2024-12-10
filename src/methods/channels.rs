use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::{database::channels::Channel, socket::RpcClient},
};

use super::{authentication::check_authenticated, Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelMethod {
    id: String,
    // TODO: scopes
    scope_id: Option<String>,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for GetChannelMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        let user = check_authenticated(clients, &id)?;
        let channel = Channel::get(&self.id).await?;
        match channel {
            Channel::PrivateChannel { .. } | Channel::GroupChannel { .. } => {
                if self.space_id.is_some() {
                    return Err(Error::NotFound);
                }
                let in_channel = user.in_channel(&channel).await?;
                if !in_channel {
                    return Err(Error::NotFound);
                }
                Ok(Response::GetChannel(GetChannelResponse { channel }))
            }
            Channel::InformationChannel { ref space_id, .. }
            | Channel::AnnouncementChannel { ref space_id, .. }
            | Channel::ChatChannel { ref space_id, .. } => {
                if let Some(request_space_id) = &self.space_id {
                    if request_space_id != space_id {
                        return Err(Error::NotFound);
                    }
                    let user_in_space = user.in_space(space_id).await?;
                    if !user_in_space {
                        return Err(Error::NotFound);
                    }
                    Ok(Response::GetChannel(GetChannelResponse { channel }))
                } else {
                    Err(Error::NotFound)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelResponse {
    channel: Channel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelsMethod {
    // TODO: work out how scopes work with private channels
    scope_id: Option<String>,
}

#[async_trait]
impl Respond for GetChannelsMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        let user = check_authenticated(clients, &id)?;
        let channels = user.get_channels().await?;
        Ok(Response::GetChannels(GetChannelsResponse { channels }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelsResponse {
    channels: Vec<Channel>,
}
// TODO: Partial structs

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateChannelMethod {
    channel: ChannelInformation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChannelInformation {
    PrivateChannel {
        target_id: String,
        scope_id: Option<String>,
    },
    GroupChannel {
        scope_id: String,
        name: String,
        description: Option<String>,
    },
    InformationChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
    AnnouncementChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
    ChatChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditChannelMethod {
    channel_id: String,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteChannelMethod {
    channel_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToChannelMethod {
    channel_id: String,
    user_id: String,
}
