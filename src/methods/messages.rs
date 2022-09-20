use std::sync::Arc;

use async_std::sync::Mutex;
use dashmap::DashMap;

use crate::services::socket::VoiceClient;

use super::{GetChannelMessagesMethod, GetChannelMessagesResponse, SendChannelMessageMethod, SendChannelMessageResponse, Response, ErrorResponse};

pub async fn get_channel_messages(m: GetChannelMessagesMethod) -> Response {
    let messages = crate::services::database::messages::get_messages(m.channel_id).await;
    Response::GetChannelMessages(GetChannelMessagesResponse { messages })
}

pub async fn send_channel_message(
    m: SendChannelMessageMethod,
    clients: Arc<Mutex<DashMap<String, VoiceClient>>>,
    id: String,
) -> Response {
    let trimmed = m.content.trim();
    if trimmed.len() > 4096 {
        return Response::Error(ErrorResponse {
            error: "Message too long".to_string(),
        });
    }
    if trimmed.len() < 1 {
        return Response::Error(ErrorResponse {
            error: "Message cannot be empty".to_string(),
        });
    }
    let clients_locked = clients.lock().await;
    let client = clients_locked.get(&id).unwrap();
    let message = crate::services::database::messages::create_message(m.channel_id, client.get_user_id(), trimmed.to_owned()).await;
    Response::SendChannelMessage(SendChannelMessageResponse { message_id: message.id })
}
