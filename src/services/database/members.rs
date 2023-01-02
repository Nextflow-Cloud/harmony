use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::permissions::{Permission, PermissionSet},
};

use super::{channels::Channel, roles::Role, spaces::Space};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub space_id: String,
    pub roles: Vec<String>,
}

impl Member {
    pub async fn get_permissions(&self) -> Result<PermissionSet> {
        let space = Space::get(&self.space_id).await?;
        if space.owner == self.id {
            Ok(PermissionSet::all())
        } else {
            let mut calculated_permissions = PermissionSet::from(space.base_permissions);
            let roles_sorted = self.roles.clone();
            let futures = roles_sorted.iter().map(Role::get);
            let mut roles = futures_util::future::try_join_all(futures).await?;
            roles.sort_by(|a, b| a.position.cmp(&b.position));
            roles.reverse();
            let default = calculated_permissions.to_vec();
            for role in roles {
                let role_permissions: PermissionSet = role.permissions.into();
                if role_permissions.has_permission(Permission::Administrator) {
                    calculated_permissions = PermissionSet::all();
                    break;
                }
                for permission in &default {
                    if !role_permissions.has_permission(*permission) {
                        calculated_permissions.remove_permission(*permission);
                    }
                }
                calculated_permissions.combine(role_permissions);
            }
            Ok(calculated_permissions)
        }
    }

    pub async fn get_channel_permissions(&self, channel: &Channel) -> Result<PermissionSet> {
        match channel {
            Channel::PrivateChannel { .. } | Channel::GroupChannel { .. } => Err(Error::NotFound),
            // FIXME: This is a temporary solution
            Channel::InformationChannel {
                id, permissions, ..
            }
            | Channel::AnnouncementChannel {
                id, permissions, ..
            }
            | Channel::ChatChannel {
                id, permissions, ..
            } => {
                // let permissions_space = self.get_permissions().await?;
                todo!("{} {:?}", id, permissions)
            }
        }
    }

    pub async fn is_owner(&self) -> Result<bool> {
        let space = Space::get(&self.space_id).await?;
        Ok(space.owner == self.id)
    }
}

pub async fn delete_member(member_id: String, space_id: String) -> Result<()> {
    let database = super::get_database();
    database
        .collection::<Member>("members")
        .delete_one(
            doc! {
                "id": member_id,
                "space_id": space_id
            },
            None,
        )
        .await?;
    Ok(())
}

pub async fn get_member(member_id: String, space_id: String) -> Result<Member> {
    let database = super::get_database();
    let member = database
        .collection::<Member>("members")
        .find_one(
            doc! {
                "id": member_id,
                "space_id": space_id,
            },
            None,
        )
        .await?
        .ok_or(crate::errors::Error::NotFound)?;
    Ok(member)
}
