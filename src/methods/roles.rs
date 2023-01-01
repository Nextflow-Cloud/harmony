use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{services::{database::{roles::{Role, create_role, Color, update_role, get_role, delete_role}, members::get_member}, socket::RpcClient, permissions::{can_modify_role, Permission}}, errors::Error};

use super::{Respond, Response, ErrorResponse};

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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let role = create_role(self.space_id.clone(), self.name.clone(), self.permissions, self.color.clone()).await;
        match role {
            Ok(role) => Response::CreateRole(CreateRoleResponse { role }),
            Err(error) => Response::Error(ErrorResponse { error })
        }
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let role = get_role(self.id.clone()).await;
        let role = match role {
            Ok(role) => role,
            Err(error) => return Response::Error(ErrorResponse { error })
        };
        if role.space_id != self.space_id {
            return Response::Error(ErrorResponse {
                error: Error::NotFound
            });
        }
        if let Some(scope_id) = &self.scope_id {
            if role.scope_id != scope_id.clone() {
                return Response::Error(ErrorResponse {
                    error: Error::NotFound
                });
            }
        }
        let member = get_member(id, self.space_id.clone()).await;
        let member = match member {
            Ok(member) => member,
            Err(error) => return Response::Error(ErrorResponse { error })
        };
        let can_modify = can_modify_role(member, role).await;
        if !can_modify {
            return Response::Error(ErrorResponse {
                error: Error::MissingPermission {
                    permission: Permission::ManageRoles
                }
            });
        }
        let role = update_role(self.id.clone(), self.name.clone(), self.permissions, self.color.clone()).await;
        match role {
            Ok(role) => Response::EditRole(EditRoleResponse { role }),
            Err(error) => Response::Error(ErrorResponse { error })
        }
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let role = delete_role(self.id.clone()).await;
        match role {
            Ok(_) => Response::DeleteRole(DeleteRoleResponse { id: self.id.clone() }),
            Err(error) => Response::Error(ErrorResponse { error })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoleResponse {
    id: String,
}
