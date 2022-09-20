use std::sync::Arc;

use async_std::sync::Mutex;
use dashmap::DashMap;

use crate::services::socket::VoiceClient;

use super::{GetChannelMessagesMethod, GetChannelMessagesResponse, SendChannelMessageMethod, SendChannelMessageResponse, Response};

pub async fn get_channel_messages(m: GetChannelMessagesMethod) -> Response {
    let messages = crate::services::database::messages::get_messages(m.channel_id).await;
    Response::GetChannelMessages(GetChannelMessagesResponse { messages })
}

pub async fn send_channel_message(
    m: SendChannelMessageMethod,
    clients: Arc<Mutex<DashMap<String, VoiceClient>>>,
    id: String,
) -> Response {
    let clients_locked = clients.lock().await;
    let client = clients_locked.get(&id).unwrap();
    let message = crate::services::database::messages::create_message(m.channel_id, client.get_user_id(), m.content).await;
    Response::SendChannelMessage(SendChannelMessageResponse { message_id: message.id })
}
