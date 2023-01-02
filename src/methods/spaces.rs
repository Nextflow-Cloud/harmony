use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::{
        database::spaces::{Space},
        socket::RpcClient,
    },
};

use super::{Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSpaceMethod {
    space_id: String,
}

#[async_trait]
impl Respond for GetSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        let user = super::authentication::check_authenticated(&clients, &id)?;
        let space = Space::get(&self.space_id).await?;
        let user_in_space = user.in_space(&self.space_id).await?;
        if !user_in_space {
            return Err(Error::NotFound);
        }
        Ok(Response::GetSpace(GetSpaceResponse { space }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        super::authentication::check_authenticated(&clients, &id)?;
        let trimmed = self.name.trim();
        if trimmed.len() > 32 {
            return Err(Error::NameTooLong);
        }
        if trimmed.is_empty() {
            return Err(Error::NameEmpty);
        }
        let space = Space::create(
            self.name.clone(),
            self.description.clone(),
            id,
            self.scope.clone(),
        )
        .await?;
        Ok(Response::CreateSpace(CreateSpaceResponse { space }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        let user = super::authentication::check_authenticated(&clients, &id)?;
        let space = user.accept_invite(&self.code).await?;
        space.add_member(&id).await?;
        Ok(Response::JoinSpace(JoinSpaceResponse { space }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        let user = super::authentication::check_authenticated(&clients, &id)?;
        let user_in_space = user.in_space(&self.space_id).await?;
        if !user_in_space {
            return Err(Error::NotFound);
        }
        let space = Space::get(&self.space_id).await?;
        space.remove_member(&id).await?;
        Ok(Response::LeaveSpace(LeaveSpaceResponse {
            space_id: self.space_id.clone(),
        }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        let user = super::authentication::check_authenticated(&clients, &id)?;
        let spaces = user.get_spaces().await?;
        Ok(Response::GetSpaces(GetSpacesResponse { spaces }))
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
// TODO: logger
#[async_trait]
impl Respond for EditSpaceMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        super::authentication::check_authenticated(&clients, &id)?;
        let space = Space::get(&self.space_id).await?;
        let space = space.update(
            self.name.clone(),
            self.description.clone(),
            self.base_permissions,
        )
        .await?;
        Ok(Response::EditSpace(EditSpaceResponse { space }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        super::authentication::check_authenticated(&clients, &id)?;
        let space = Space::get(&self.space_id).await?;
        space.delete().await?;
        Ok(Response::DeleteSpace(DeleteSpaceResponse {
            id: self.space_id.clone(),
        }))
    }
}

// FIXME: sudo mode

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSpaceResponse {
    id: String,
}
