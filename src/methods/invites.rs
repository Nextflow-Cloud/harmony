use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{services::{socket::RpcClient, database::{invites::{create_invite, Invite, delete_invite, get_invite, get_invites}, members::get_member, spaces::get_space, infractions::is_banned, channels::{get_channel, Channel}}, permissions::{permissions_for, Permission}}, errors::Error};

use super::{Respond, Response, ErrorResponse};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateInviteMethod {
    channel_id: String,
    max_uses: Option<i32>,
    expires_at: Option<u64>,
    authorized_users: Option<Vec<String>>,
    space_id: Option<String>,
    scope_id: Option<String>,
}

#[async_trait]
impl Respond for CreateInviteMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let user_id = client.get_user_id();
        let invite = create_invite(self.channel_id.clone(), user_id, self.expires_at, self.max_uses, self.authorized_users.clone(), self.space_id.clone(), self.scope_id.clone()).await;
        match invite {
            Ok(invite) => Response::CreateInvite(CreateInviteResponse { invite }),
            Err(error) => Response::Error(ErrorResponse { error })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateInviteResponse {
    invite: Invite,
}

// pub struct UpdateInviteMethod {
//     code: String,
//     max_uses: Option<i32>,
//     expires_at: Option<u64>,
// }

// #[async_trait]
// impl Respond for UpdateInviteMethod {
//     async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
//         let client = clients.get(&id.clone()).unwrap();
//         let member = get_member(id).await;
//         match member {
//             Ok(member) => {
//                 let permissions = permissions_for(member).await;
//                 if !permissions.has_permission(Permission::ManageInvites) {
//                     return Response::Error(ErrorResponse { error: "You do not have permission to manage invites".to_string() });
//                 } else {
                    
//                 }
//             },
//             Err(error) => Response::Error(ErrorResponse { error })
//         }
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteInviteMethod {
    id: String,
    space_id: String,
}

#[async_trait]
impl Respond for DeleteInviteMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let user_id = client.get_user_id();
        let member = get_member(user_id, self.space_id.clone()).await;
        match member {
            Ok(member) => {
                let permissions = permissions_for(member).await;
                if !permissions.has_permission(Permission::ManageInvites) {
                    return Response::Error(ErrorResponse { error: Error::MissingPermission {
                        permission: Permission::ManageInvites
                    } });
                } else {
                    let result = delete_invite(self.id.clone()).await;
                    match result {
                        Ok(_) => Response::DeleteInvite(DeleteInviteResponse {}),
                        Err(error) => Response::Error(ErrorResponse { error })
                    }
                }
            },
            Err(error) => Response::Error(ErrorResponse { error })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteInviteResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteMethod {
    code: String,
}

#[async_trait]
impl Respond for GetInviteMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let user_id = client.get_user_id();
        let invite = get_invite(self.code.clone()).await;
        match invite {
            Ok(invite) => {
                if let Some(space_id) = invite.space_id {
                    let space = get_space(space_id).await;
                    match space {
                        Ok(space) => {
                            let banned = is_banned(user_id.clone(), space.id).await;
                            match banned {
                                Ok(banned) => {
                                    Response::GetInvite(GetInviteResponse {
                                        invite: InviteInformation::Space {
                                            name: space.name,
                                            description: space.description,
                                            inviter_id: invite.creator,
                                            banned,
                                            authorized: invite.authorized_users.unwrap_or(vec![user_id.clone()]).contains(&user_id),
                                            member_count: space.members.len() as i32,
                                        }
                                    })
                                },
                                Err(error) => Response::Error(ErrorResponse { error })
                            }
                        },
                        Err(error) => Response::Error(ErrorResponse { error })
                    }
                } else {
                    let channel = get_channel(invite.channel_id).await;
                    match channel {
                        Ok(channel) => {
                            if let Channel::GroupChannel { name, description, members, .. } = channel {
                                Response::GetInvite(GetInviteResponse {
                                    invite: InviteInformation::Group {
                                        name,
                                        description,
                                        inviter_id: invite.creator,
                                        authorized: invite.authorized_users.unwrap_or(vec![user_id.clone()]).contains(&user_id),
                                        member_count: members.len() as i32,
                                    }
                                })
                            } else {
                                Response::Error(ErrorResponse { error: Error::InvalidInvite })
                            }
                        },
                        Err(error) => Response::Error(ErrorResponse { error })
                    }
                }
            },
            Err(error) => Response::Error(ErrorResponse { error })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InviteInformation {
    #[serde(rename_all = "camelCase")]
    Group {
        name: String,
        description: String,
        inviter_id: String,
        authorized: bool,
        member_count: i32,
    }, 
    #[serde(rename_all = "camelCase")]
    Space {
        name: String,
        description: String,
        inviter_id: String,
        banned: bool,
        authorized: bool,
        member_count: i32,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInviteResponse {
    #[serde(flatten)]
    invite: InviteInformation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInvitesMethod {
    channel_id: String,
    space_id: Option<String>,
    scope_id: Option<String>,
}

#[async_trait]
impl Respond for GetInvitesMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let client = clients.get(&id).unwrap();
        let user_id = client.get_user_id();
        if let Some(space_id) = &self.space_id {
            let member = get_member(user_id, space_id.clone()).await;
            match member {
                Ok(member) => {
                    let permissions = permissions_for(member).await;
                    if !permissions.has_permission(Permission::ManageInvites) {
                        return Response::Error(ErrorResponse { error: Error::MissingPermission {
                            permission: Permission::ManageInvites
                        } });
                    }
                },
                Err(error) => return Response::Error(ErrorResponse { error })
            };
        }
        let invites = get_invites(self.channel_id.clone(), self.space_id.clone()).await;
        match invites {
            Ok(invites) => Response::GetInvites(GetInvitesResponse { invites }),
            Err(error) => Response::Error(ErrorResponse { error })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInvitesResponse {
    invites: Vec<Invite>,
}

// TODO: Invite manager built-in
