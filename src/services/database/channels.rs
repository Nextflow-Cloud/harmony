use futures_util::{StreamExt};
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::permissions::PermissionSet,
};

use super::messages::Message;

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

impl Channel {
    pub async fn get(id: &String) -> Result<Channel> {
    let database = super::get_database();
    let channel = database
        .collection::<Channel>("channels")
        .find_one(
            doc! {
                    "id": id,
            },
            None,
        )
        .await?;
    match channel {
        Some(channel) => Ok(channel),
        None => Err(Error::NotFound),
    }
}
    pub async fn get_messages(
        &self,
        limit: Option<i64>,
        latest: Option<bool>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<Vec<Message>> {
        match self {
            Channel::AnnouncementChannel { id, .. }
            | Channel::ChatChannel { id, .. } => {
    let database = super::get_database();
                let limit = limit.unwrap_or(50);
                let mut query = doc! { "channelId": id };
                if let Some(before) = before {
                    query.insert("id", doc! { "$lt": before });
}
                if let Some(after) = after {
                    query.insert("id", doc! { "$gt": after });
            }
                let options = FindOptions::builder()
                    .sort(doc! {
                        "id": if latest.unwrap_or(false) { -1 } else { 1 }
                    })
                    .limit(limit)
                    .build();
                let messages: Vec<_> = database
                    .collection::<Message>("messages")
                    .find(query, options)
                    .await?
                    .collect()
                    .await;
                let messages = messages.into_iter().map(|m| m.map_err(|e| e.into())).collect::<Result<Vec<_>>>()?;
                
                Ok(messages)
            },
            _ => Err(Error::NotFound),
            }
        }
    }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PermissionOverride {
    pub id: String,
    pub allow: PermissionSet,
    pub deny: PermissionSet,
}
