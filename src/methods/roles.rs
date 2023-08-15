use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::{
        database::{
            roles::{Color, Role},
            spaces::Space, members::Member,
        },
        permissions::{can_modify_role, Permission},
        socket::RpcClient,
    },
};

use super::{Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleMethod {
    name: String,
    permissions: i64,
    color: Color,
    space_id: String,
}

#[async_trait]
impl Respond for CreateRoleMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        let space = Space::get(&self.space_id).await?;
        let role = Role::create(
            &space,
            self.name.clone(),
            self.permissions,
            self.color.clone(),
        )
        .await?;
        Ok(Response::CreateRole(CreateRoleResponse { role }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct CreateRoleResponse {
    role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRoleMethod {
    id: String,
    name: String,
    permissions: i64,
    color: Color,
    space_id: String,
    scope_id: Option<String>,
}

#[async_trait]
impl Respond for EditRoleMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        let role = Role::get(&self.id).await?;
        if role.space_id != self.space_id {
            return Err(Error::NotFound);
        }
        if let Some(scope_id) = &self.scope_id {
            if role.scope_id != scope_id.clone() {
                return Err(Error::NotFound);
            }
        }
        let member = Member::get(&id, &self.space_id).await?;
        let can_modify = can_modify_role(&member, &role).await?;
        if !can_modify {
            return Err(Error::MissingPermission {
                permission: Permission::ManageRoles,
            });
        }
        let role = role
            .update(self.name.clone(), self.permissions, self.color.clone())
            .await?;
        Ok(Response::EditRole(EditRoleResponse { role }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRoleResponse {
    role: Role,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoleMethod {
    id: String,
}

#[async_trait]
impl Respond for DeleteRoleMethod {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response> {
        super::authentication::check_authenticated(clients, &id)?;
        let role = Role::get(&self.id).await?;
        role.delete().await?;
        Ok(Response::DeleteRole(DeleteRoleResponse {
            id: self.id.clone(),
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoleResponse {
    id: String,
}
