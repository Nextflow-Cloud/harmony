use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    services::{
        database::{
            channels::{get_channel, in_channel, Channel},
            spaces::in_space,
        },
        socket::RpcClient,
    },
};

use super::{ErrorResponse, Respond, Response};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelMethod {
    channel_id: String,
    // TODO: scopes
    scope_id: Option<String>,
    space_id: Option<String>,
}

#[async_trait]
impl Respond for GetChannelMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let channel = get_channel(self.channel_id.clone()).await;
        match channel {
            Ok(channel) => {
                let client = clients.get(&id).unwrap();
                match channel {
                    Channel::PrivateChannel { ref id, .. }
                    | Channel::GroupChannel { ref id, .. } => {
                        if self.space_id.is_some() {
                            return Response::Error(ErrorResponse {
                                error: Error::NotFound,
                            });
                        }
                        let user_in_channel = in_channel(client.get_user_id(), id.clone()).await;
                        match user_in_channel {
                            Ok(user_in_channel) => {
                                if !user_in_channel {
                                    return Response::Error(ErrorResponse {
                                        error: Error::NotFound,
                                    });
                                }
                                Response::GetChannel(GetChannelResponse { channel })
                        }
                            Err(error) => Response::Error(ErrorResponse { error }),
                        }
                    }
                    Channel::InformationChannel { ref space_id, .. }
                    | Channel::AnnouncementChannel { ref space_id, .. }
                    | Channel::ChatChannel { ref space_id, .. } => {
                        if let Some(request_space_id) = &self.space_id {
                            if request_space_id != space_id {
                                return Response::Error(ErrorResponse {
                                    error: Error::NotFound,
                                });
                            }
                            let user_in_space =
                                in_space(client.get_user_id(), space_id.clone()).await;
                            match user_in_space {
                                Ok(user_in_space) => {
                                    if !user_in_space {
                                        return Response::Error(ErrorResponse {
                                            error: Error::NotFound,
                                        });
                                    }
                                    Response::GetChannel(GetChannelResponse { channel })
                                }
                                Err(error) => Response::Error(ErrorResponse { error }),
                            }
                        } else {
                            Response::Error(ErrorResponse {
                                error: Error::NotFound,
                            })
                        }
                    }
                }
            }
            Err(error) => Response::Error(ErrorResponse { error }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelResponse {
    channel: Channel,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GetChannelsMethod {
//     scope_id: Option<String>,
// }
// TODO: work out how scopes work with private channels

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateChannelMethod {
    channel: ChannelInformation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChannelInformation {
    PrivateChannel {
        target_id: String,
        scope_id: Option<String>,
    },
    GroupChannel {
        scope_id: String,
        name: String,
        description: Option<String>,
    },
    InformationChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
    AnnouncementChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
    ChatChannel {
        space_id: String,
        scope_id: Option<String>,
        name: String,
        description: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditChannelMethod {
    channel_id: String,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct DeleteChannelMethod {
    channel_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToChannelMethod {
    channel_id: String,
    user_id: String,
}
