use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::{database::messages::Message, socket::RpcClient},
};

use self::{
    authentication::{
        GetIdMethod, GetIdResponse, HeartbeatMethod, HeartbeatResponse, IdentifyMethod,
        IdentifyResponse,
    },
    channels::{GetChannelMethod, GetChannelResponse, GetChannelsMethod, GetChannelsResponse},
    invites::{
        CreateInviteMethod, CreateInviteResponse, DeleteInviteMethod, DeleteInviteResponse,
        GetInviteMethod, GetInviteResponse, GetInvitesMethod, GetInvitesResponse,
    },
    messages::{GetMessagesMethod, GetMessagesResponse, SendMessageMethod, SendMessageResponse},
    roles::{
        CreateRoleMethod, CreateRoleResponse, DeleteRoleMethod, DeleteRoleResponse, EditRoleMethod,
        EditRoleResponse,
    },
    spaces::{
        CreateSpaceMethod, CreateSpaceResponse, DeleteSpaceMethod, DeleteSpaceResponse,
        EditSpaceMethod, EditSpaceResponse, GetSpaceMethod, GetSpaceResponse, GetSpacesResponse,
        JoinSpaceResponse, LeaveSpaceResponse,
    },
    webrtc::{
        EndCallMethod, EndCallResponse, JoinCallMethod, JoinCallResponse, LeaveCallMethod,
        LeaveCallResponse, StartCallMethod, StartCallResponse,
    },
};

pub mod authentication;
pub mod channels;
pub mod events;
pub mod invites;
pub mod messages;
pub mod roles;
pub mod spaces;
pub mod users;
pub mod webrtc;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i8)]
pub enum Method {
    Identify(IdentifyMethod) = 1,
    Heartbeat(HeartbeatMethod) = 2,
    GetId(GetIdMethod) = 3,

    // WebRTC: 10-19
    StartCall(StartCallMethod) = 10,
    JoinCall(JoinCallMethod) = 11,
    LeaveCall(LeaveCallMethod) = 12,
    EndCall(EndCallMethod) = 13,

    GetMessages(GetMessagesMethod) = 20,
    SendMessage(SendMessageMethod) = 22,

    GetChannel(GetChannelMethod) = 30,
    GetChannels(GetChannelsMethod) = 31,
    // CreateChannel(CreateChannelMethod) = 32,
    // EditChannel(EditChannelMethod) = 33,
    // DeleteChannel(DeleteChannelMethod) = 34,
    GetSpace(GetSpaceMethod) = 40,
    CreateSpace(CreateSpaceMethod) = 41,
    EditSpace(EditSpaceMethod) = 42,
    DeleteSpace(DeleteSpaceMethod) = 43,

    // AddFriend(AddFriendMethod) = 50,
    // RemoveFriend(RemoveFriendMethod) = 51,
    // GetFriends(GetFriendsMethod) = 52,
    // GetFriendRequests(GetFriendRequestsMethod) = 53,
    // AcknowledgeFriendRequest(AcknowledgeFriendRequestMethod) = 55,
    CreateInvite(CreateInviteMethod) = 60,
    DeleteInvite(DeleteInviteMethod) = 61,
    GetInvite(GetInviteMethod) = 62,
    GetInvites(GetInvitesMethod) = 63,

    CreateRole(CreateRoleMethod) = 70,
    EditRole(EditRoleMethod) = 71,
    DeleteRole(DeleteRoleMethod) = 72,
    // GetRoles(GetRolesMethod) = 73,
}

#[async_trait]
pub trait Respond {
    async fn respond(
        &self,
        clients: Arc<DashMap<String, RpcClient>>,
        id: String,
    ) -> Result<Response>;
}

pub fn get_respond(m: Method) -> Box<dyn Respond + Send + Sync> {
    match m {
        Method::Identify(m) => Box::new(m),
        Method::Heartbeat(m) => Box::new(m),
        Method::GetId(m) => Box::new(m),
        Method::StartCall(m) => Box::new(m),
        Method::JoinCall(m) => Box::new(m),
        Method::LeaveCall(m) => Box::new(m),
        Method::EndCall(m) => Box::new(m),
        Method::GetMessages(m) => Box::new(m),
        Method::SendMessage(m) => Box::new(m),
        Method::GetChannel(m) => Box::new(m),
        Method::GetChannels(m) => Box::new(m),
        // Method::CreateChannel(m) => m,
        // Method::EditChannel(m) => m,
        // Method::DeleteChannel(m) => m,
        Method::GetSpace(m) => Box::new(m),
        Method::CreateSpace(m) => Box::new(m),
        Method::EditSpace(m) => Box::new(m),
        Method::DeleteSpace(m) => Box::new(m),
        // Method::AddFriend(m) => m,
        // Method::RemoveFriend(m) => m,
        // Method::GetFriends(m) => m,
        // Method::GetFriendRequests(m) => m,
        // Method::AcknowledgeFriendRequest(m) => m,
        Method::CreateInvite(m) => Box::new(m),
        Method::CreateRole(m) => Box::new(m),
        Method::EditRole(m) => Box::new(m),
        Method::DeleteRole(m) => Box::new(m),
        Method::DeleteInvite(m) => Box::new(m),
        Method::GetInvite(m) => Box::new(m),
        Method::GetInvites(m) => Box::new(m),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RpcApiMethod {
    pub(crate) id: Option<String>,
    #[serde(flatten)]
    pub(crate) method: Method,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddFriendMethod {
    channel_id: String,
    friend_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFriendMethod {
    channel_id: String,
    friend_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFriendsMethod {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFriendRequestsMethod {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeFriendRequestMethod {
    channel_id: String,
    friend_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(i8)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Response {
    Identify(IdentifyResponse) = 1,
    Heartbeat(HeartbeatResponse) = 2,
    GetId(GetIdResponse) = 3,

    // WebRTC: 10-19
    StartCall(StartCallResponse) = 10,
    JoinCall(JoinCallResponse) = 11,
    LeaveCall(LeaveCallResponse) = 12,
    EndCall(EndCallResponse) = 13,

    GetMessages(GetMessagesResponse) = 20,
    SendMessage(SendMessageResponse) = 22,

    GetChannel(GetChannelResponse) = 30,
    GetChannels(GetChannelsResponse) = 31,
    // CreateChannel(CreateChannelResponse) = 32,
    // EditChannel(EditChannelResponse) = 33,
    // DeleteChannel(DeleteChannelResponse) = 34,
    GetSpace(GetSpaceResponse) = 40,
    CreateSpace(CreateSpaceResponse) = 41,
    EditSpace(EditSpaceResponse) = 42,
    DeleteSpace(DeleteSpaceResponse) = 43,
    JoinSpace(JoinSpaceResponse) = 44,
    LeaveSpace(LeaveSpaceResponse) = 45,
    GetSpaces(GetSpacesResponse) = 46,

    CreateInvite(CreateInviteResponse) = 60,
    DeleteInvite(DeleteInviteResponse) = 61,
    GetInvite(GetInviteResponse) = 62,
    GetInvites(GetInvitesResponse) = 63,

    CreateRole(CreateRoleResponse) = 70,
    EditRole(EditRoleResponse) = 71,
    DeleteRole(DeleteRoleResponse) = 72,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcApiResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response: Option<Response>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<Error>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(i8)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    Hello(HelloEvent) = 0,

    // WebRTC: 10-19
    NewMessage(NewMessageEvent) = 21,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcApiEvent {
    #[serde(flatten)]
    pub(crate) event: Event,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloEvent {
    pub(crate) public_key: Vec<u8>,
    pub(crate) request_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageEvent {
    message: Message,
    channel_id: String,
}

pub enum CreateChannelType {
    PrivateChannel {
        peer_id: String,
        scope_id: String,
    },
    GroupChannel {
        name: String,
        description: String,
        members: Vec<String>,
        scope_id: String,
    },
    InformationChannel {
        name: String,
        description: String,
        nexus_id: String,
        scope_id: String,
    },
    TextChannel {
        name: String,
        description: String,
        nexus_id: String,
        scope_id: String,
    },
}
