use futures_util::TryStreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::{Error, Result};

use super::{channels::Channel, invites::Invite, members::Member, roles::Role};
// use super::invites::Invite;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    pub id: String,
    pub name: String,
    pub description: String,
    pub channels: Vec<String>,
    pub members: Vec<String>,
    pub roles: Vec<String>,
    pub owner: String,
    pub scope_id: String,
    pub base_permissions: i64,
}

impl Space {
    pub async fn create(
        name: String,
        description: Option<String>,
        owner: String,
        scope_id: Option<String>,
    ) -> Result<Space> {
        let spaces = super::get_database().collection::<Space>("spaces");
        let space = Space {
            id: Ulid::new().to_string(),
            name,
            description: description.unwrap_or("".to_string()),
            channels: Vec::new(),
            members: Vec::new(),
            roles: Vec::new(),
            owner,
            scope_id: scope_id.unwrap_or("global".to_owned()),
            base_permissions: 0x16,
        };
        spaces.insert_one(space.clone(), None).await?;
        Ok(space)
    }

    pub async fn delete(&self) -> Result<()> {
        let spaces = super::get_database().collection::<Space>("spaces");
        spaces
            .delete_one(
                doc! {
                    "id": &self.id,
                },
                None,
            )
            .await?;
        let channels = super::get_database().collection::<Channel>("channels");
        channels
            .delete_many(
                doc! {
                    "space_id": &self.id,
                },
                None,
            )
            .await?;
        let invites = super::get_database().collection::<Invite>("invites");
        invites
            .delete_many(
                doc! {
                    "space_id": &self.id,
                },
                None,
            )
            .await?;
        let roles = super::get_database().collection::<Role>("roles");
        roles
            .delete_many(
                doc! {
                    "space_id": &self.id,
                },
                None,
            )
            .await?;
        let members = super::get_database().collection::<Member>("members");
        members
            .delete_many(
                doc! {
                    "space_id": &self.id,
                },
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn get(id: &String) -> Result<Space> {
        let spaces = super::get_database().collection::<Space>("spaces");
        let space = spaces
            .find_one(
                doc! {
                    "id": id,
                },
                None,
            )
            .await?;
        if let Some(space) = space {
            Ok(space)
        } else {
            Err(Error::NotFound)
        }
    }
    pub async fn add_member(&self, id: &String) -> Result<()> {
        let spaces = super::get_database().collection::<Space>("spaces");
        spaces
            .update_one(
                doc! {
                    "id": &self.id,
                },
                doc! {
                    "$push": {
                        "members": id,
                    },
                },
                None,
            )
            .await?;
        Ok(())
    }
    pub async fn remove_member(&self, id: &String) -> Result<()> {
        let spaces = super::get_database().collection::<Space>("spaces");
        spaces
            .update_one(
                doc! {
                    "id": &self.id,
                },
                doc! {
                    "$pull": {
                        "members": id,
                    },
                },
                None,
            )
            .await?;
        Ok(())
    }
    pub async fn update(
        &self,
        name: Option<String>,
        description: Option<String>,
        base_permissions: Option<i32>,
    ) -> Result<Space> {
        let spaces = super::get_database().collection::<Space>("spaces");
        let mut update = doc! {};
        if let Some(name) = name {
            update.insert("name", name);
        }
        if let Some(description) = description {
            update.insert("description", description);
        }
        if let Some(base_permissions) = base_permissions {
            update.insert("base_permissions", base_permissions);
        }
        let space = spaces
            .find_one_and_update(
                doc! {
                    "id": &self.id,
                },
                doc! {
                    "$set": update,
                },
                None,
            )
            .await?;
        match space {
            Some(space) => Ok(space),
            None => Err(Error::NotFound),
        }
    }
    pub async fn change_owner(&self, user_id: &String) -> Result<()> {
        let spaces = super::get_database().collection::<Space>("spaces");
        spaces
            .update_one(
                doc! {
                    "id": &self.id,
                },
                doc! {
                    "$set": {
                        "owner": user_id,
                    },
                },
                None,
            )
            .await?;
        Ok(())
    }
    
    pub async fn get_channels(&self) -> Result<Vec<Channel>> {
        let database = super::get_database();
        let channels: Vec<Channel> = database
            .collection::<Channel>("channels")
            .find(
                doc! {
                    "spaceId": &self.id,
                },
                None,
            )
            .await?
            .try_collect()
            .await?;
        Ok(channels)
    }
}
