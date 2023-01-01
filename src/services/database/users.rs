use mongodb::{bson::doc, error::Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Status {
    Online = 0,
    Idle = 1,
    Busy = 2,
    Invisible = 3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Presence {
    status: Status,
    message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Relationship {
    Friend = 0,
    Blocked = 1,
    Pending = 2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Affinity {
    id: String,
    relationship: Relationship,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    id: String,
    profile_banner: String, // TODO: Make use of file handling
    profile_description: String,
    presence: Presence,
    online: bool,
    // usernames on Nextflow are unique
    // can set a display name for better visibility
    platform_administrator: bool, // TODO: should be implemented globally
                                  // on SSO system user data
    affinities: Vec<Affinity>,
}

pub async fn get_user(id: String) -> Result<Option<User>, Error> {
    let users = super::get_database().collection::<User>("users");
    users
        .find_one(
            doc! {
                "id": id
            },
            None,
        )
        .await
}
