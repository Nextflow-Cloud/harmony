use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    services::{
        database::spaces::{delete_space, update_space, Space},
        socket::RpcClient,
    },
};

use super::{ErrorResponse, Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpaceMethod {
    space_id: String,
}

#[async_trait]
impl Respond for GetSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let space = crate::services::database::spaces::get_space(self.space_id.clone()).await;
        match space {
            Ok(space) => {
                let user_in_space = crate::services::database::spaces::in_space(
                    client.get_user_id(),
                    self.space_id.clone(),
                )
                .await;
                match user_in_space {
                    Ok(user_in_space) => {
                        if !user_in_space {
                            return Response::Error(ErrorResponse {
                                error: Error::NotFound,
                            });
                        }
                        Response::GetSpace(GetSpaceResponse { space })
                    }
                    Err(error) => Response::Error(ErrorResponse { error }),
                }
            }
            Err(error) => Response::Error(ErrorResponse { error }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpaceResponse {
    space: Space,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceMethod {
    // space: Space,
    name: String,
    description: Option<String>,
    scope: Option<String>,
}

#[async_trait]
impl Respond for CreateSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let trimmed = self.name.trim();
        if trimmed.len() > 32 {
            return Response::Error(ErrorResponse {
                error: Error::NameTooLong,
            });
        }
        if trimmed.len() < 1 {
            return Response::Error(ErrorResponse {
                error: Error::NameEmpty,
            });
        }
        let client = clients.get(&id).unwrap();
        let space = crate::services::database::spaces::create_space(
            self.name.clone(),
            self.description.clone(),
            client.get_user_id(),
            self.scope.clone(),
        )
        .await;
        match space {
            Ok(space) => Response::CreateSpace(CreateSpaceResponse { space }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceResponse {
    space: Space,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinSpaceMethod {
    code: String,
}

#[async_trait]
impl Respond for JoinSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let space = crate::services::database::invites::accept_invite(
            client.get_user_id(),
            self.code.clone(),
        )
        .await;
        match space {
            Ok(space) => Response::JoinSpace(JoinSpaceResponse { space }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinSpaceResponse {
    space: Space,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveSpaceMethod {
    space_id: String,
}

#[async_trait]
impl Respond for LeaveSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let space = crate::services::database::spaces::leave_space(
            self.space_id.clone(),
            client.get_user_id(),
        )
        .await;
        match space {
            Ok(_) => Response::LeaveSpace(LeaveSpaceResponse {
                space_id: self.space_id.clone(),
            }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveSpaceResponse {
    space_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpacesMethod {}

#[async_trait]
impl Respond for GetSpacesMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let spaces = crate::services::database::spaces::get_spaces(client.get_user_id()).await;
        match spaces {
            Ok(spaces) => Response::GetSpaces(GetSpacesResponse { spaces }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpacesResponse {
    spaces: Vec<Space>,
}

// FIXME: permissions

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSpaceMethod {
    space_id: String,
    name: Option<String>,
    description: Option<String>,
    base_permissions: Option<i32>,
}

#[async_trait]
impl Respond for EditSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let space = update_space(
            self.space_id.clone(),
            self.name.clone(),
            self.description.clone(),
            self.base_permissions,
        )
        .await;
        match space {
            Ok(space) => Response::EditSpace(EditSpaceResponse { space }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSpaceResponse {
    space: Space,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSpaceMethod {
    space_id: String,
}

#[async_trait]
impl Respond for DeleteSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let space = delete_space(self.space_id.clone()).await;
        match space {
            Ok(_) => Response::DeleteSpace(DeleteSpaceResponse {
                id: self.space_id.clone(),
            }),
            Err(e) => Response::Error(ErrorResponse { error: e }),
        }
    }
}

// FIXME: sudo mode

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSpaceResponse {
    id: String,
}
