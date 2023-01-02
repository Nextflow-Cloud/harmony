use futures_util::TryStreamExt;
use mongodb::bson::{self, doc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub permissions: i64,
    pub color: Color,
    pub position: i32,
    pub space_id: String,
    pub scope_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Color {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

impl Role {
    pub async fn create(
        space_id: String,
        name: String,
        permissions: i64,
        color: Color,
    ) -> Result<Role> {
        let roles = super::get_database().collection::<Role>("roles");
        let space_roles = get_roles(space_id.clone()).await?;
        let position = space_roles.len() as i32;
        let role = Role {
            id: Ulid::new().to_string(),
            name,
            permissions,
            color,
            position,
            space_id,
            scope_id: "global".to_string(), // FIXME: scope_id
        };
        roles.insert_one(role.clone(), None).await?;
        Ok(role)
    }

    pub async fn delete(&self) -> Result<()> {
        let roles = super::get_database().collection::<Role>("roles");
        roles
            .delete_one(
                doc! {
                    "id": &self.id,
                },
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn get(id: &String) -> Result<Role> {
        let roles = super::get_database().collection::<Role>("roles");
        let role = roles
            .find_one(
                doc! {
                    "id": &id,
                },
                None,
            )
            .await?;
        match role {
            Some(role) => Ok(role),
            None => Err(Error::NotFound),
        }
    }

    pub async fn update(
        &self,
        name: String,
        permissions: i64,
        color: Color,
    ) -> Result<Role> {
        let roles = super::get_database().collection::<Role>("roles");
        let role = roles
            .find_one_and_update(
                doc! {
                    "id": &self.id,
                },
                doc! {
                    "$set": {
                        "name": name,
                        "permissions": permissions,
                        "color": bson::to_bson(&color)?,
                    },
                },
                None,
            )
            .await?;
        match role {
            Some(role) => Ok(role),
            None => Err(Error::NotFound),
        }
    }
}

pub async fn get_roles(space_id: String) -> Result<Vec<Role>> {
    let roles = super::get_database().collection::<Role>("roles");
    let roles = roles
        .find(
            doc! {
                "space_id": space_id,
            },
            None,
        )
        .await?;
    Ok(roles.try_collect().await?)
}
