use mongodb::{bson::doc, error::Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Channel {
    PrivateChannel {
        id: String,
        members: Vec<String>, // array of member ids
        scope_id: String // scope: "global" or id
    },
    GroupChannel {
        id: String,
        name: String,
        description: String,
        members: Vec<String>, 
        scope_id: String
    },
    InformationChannel {
        id: String,
        name: String,
        description: String,
        nexus_id: String,
        scope_id: String
    },
    ChatChannel {
        id: String,
        name: String,
        description: String,
        nexus_id: String,
        // TODO: permissions
        scope_id: String
    }
}

pub async fn get_channel(channel_id: String) -> Result<Option<Channel>, Error> {
    let database = super::get_database();
    let channel = database.collection::<Channel>("channels").find_one(doc! {
        "id": channel_id,
    }, None).await;
    channel
}
