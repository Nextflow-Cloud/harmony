use futures_util::TryStreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::permissions::PermissionSet,
};

use super::spaces::in_space;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Channel {
    PrivateChannel {
        id: String,
        initiator_id: String,
        target_id: String,
        scope_id: String, // scope: "global" or id
    },
    GroupChannel {
        id: String,
        name: String,
        description: String,
        owner_id: String,
        members: Vec<String>,
        scope_id: String,
    },
    InformationChannel {
        id: String,
        name: String,
        space_id: String,
        scope_id: String,
    },
    AnnouncementChannel {
        id: String,
        name: String,
        space_id: String,
        scope_id: String,
        permissions: PermissionOverride,
    },
    ChatChannel {
        id: String,
        name: String,
        description: String,
        space_id: String,
        scope_id: String,
        // TODO: permission checks
        permissions: PermissionOverride,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PermissionOverride {
    pub id: String,
    pub allow: PermissionSet,
    pub deny: PermissionSet,
}

pub async fn get_channel(channel_id: String) -> Result<Channel> {
    let database = super::get_database();
    let channel = database
        .collection::<Channel>("channels")
        .find_one(
            doc! {
                "id": channel_id,
            },
            None,
        )
        .await?;
    match channel {
        Some(channel) => Ok(channel),
        None => Err(Error::NotFound),
    }
}

pub async fn get_channels(space_id: String) -> Result<Vec<Channel>> {
    let database = super::get_database();
    let channels: Vec<Channel> = database
        .collection::<Channel>("channels")
        .find(
            doc! {
                "spaceId": space_id,
            },
            None,
        )
        .await?
        .try_collect()
        .await?;
    Ok(channels)
}
pub async fn in_channel(user_id: String, channel_id: String) -> Result<bool> {
    let channel = get_channel(channel_id).await?;
    match channel {
        Channel::PrivateChannel {
            initiator_id,
            target_id,
            ..
        } => {
            if initiator_id == user_id || target_id == user_id {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Channel::GroupChannel { members, .. } => {
            if members.contains(&user_id) {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Channel::InformationChannel { space_id, .. } => in_space(user_id, space_id).await,
        Channel::AnnouncementChannel { space_id, .. } => in_space(user_id, space_id).await,
        Channel::ChatChannel { space_id, .. } => in_space(user_id, space_id).await,
    }
}
