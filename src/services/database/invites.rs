use futures_util::StreamExt;
use mongodb::bson::doc;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::{Error, Result};

use super::spaces::Space;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub id: String,
    pub code: String,
    pub channel_id: String,
    pub creator: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub authorized_users: Option<Vec<String>>,
    pub space_id: Option<String>,
    pub scope_id: String,
}

pub fn generate_code() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 7)
}

pub async fn create_invite(
    channel_id: String,
    creator: String,
    expires_at: Option<u64>,
    max_uses: Option<i32>,
    authorized_users: Option<Vec<String>>,
    space_id: Option<String>,
    scope_id: Option<String>,
) -> Result<Invite> {
    let invite = Invite {
        id: Ulid::new().to_string(),
        code: generate_code(),
        channel_id,
        creator,
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        expires_at,
        max_uses,
        uses: 0,
        authorized_users,
        space_id,
        scope_id: scope_id.unwrap_or("global".to_owned()),
    };
    let database = super::get_database();
    database
        .collection::<Invite>("invites")
        .insert_one(invite.clone(), None)
        .await?;
    Ok(invite)
}

pub async fn get_invite(code: String) -> Result<Invite> {
    let database = super::get_database();
    let invite = database
        .collection::<Invite>("invites")
        .find_one(
            doc! {
                "id": code,
            },
            None,
        )
        .await?;
    match invite {
        Some(invite) => Ok(invite),
        None => Err(Error::NotFound),
    }
}

// pub async fn update_invite(code: String) -> Option<Invite> {

// }

pub async fn delete_invite(id: String) -> Result<bool> {
    let database = super::get_database();
    let result = database
        .collection::<Invite>("invites")
        .delete_one(
            doc! {
                "id": id,
            },
            None,
        )
        .await?
        .deleted_count
        > 0;
    Ok(result)
}

pub async fn accept_invite(user_id: String, invite_code: String) -> Result<Space> {
    let invites = super::get_database().collection::<Invite>("invites");
    let spaces = super::get_database().collection::<Space>("spaces");
    let invite = invites
        .find_one(
            doc! {
                "id": invite_code,
            },
            None,
        )
        .await?;
    let invite = match invite {
        Some(invite) => invite,
        None => return Err(Error::NotFound),
    };
    let space = spaces
        .find_one(
            doc! {
                "id": invite.space_id.as_ref(),
            },
            None,
        )
        .await?;
    let mut space = match space {
        Some(space) => space,
        None => return Err(Error::NotFound),
    };
    space.members.push(user_id);
    spaces
        .update_one(
            doc! {
                "id": invite.space_id.unwrap(),
            },
            doc! {
                "$set": {
                    "members": space.members.clone(),
                },
            },
            None,
        )
        .await?;
    Ok(space)
}

pub async fn get_invites(channel_id: String, space_id: Option<String>) -> Result<Vec<Invite>> {
    let database = super::get_database();
    let mut query = doc! {
        "channel_id": channel_id,
    };
    if let Some(space_id) = space_id {
        query.insert("space_id", space_id);
    }
    let invites: std::result::Result<Vec<Invite>, _> = database
        .collection::<Invite>("invites")
        .find(query, None)
        .await?
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect();
    Ok(invites?)
}
