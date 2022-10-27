use std::{sync::Arc, time::Duration};

use async_std::{sync::Mutex, future};
use async_tungstenite::tungstenite::Message;
use dashmap::DashMap;
use futures_util::SinkExt;
use rmp_serde::Serializer;
use serde::Serialize;

use crate::services::socket::VoiceClient;

use super::{GetMessagesMethod, GetMessagesResponse, SendMessageMethod, SendMessageResponse, Response, ErrorResponse, NewMessageEvent, Event, RpcApiEvent};

pub async fn get_messages(m: GetMessagesMethod) -> Response {
    let messages = crate::services::database::messages::get_messages(m.channel_id).await;
    Response::GetMessages(GetMessagesResponse { messages })
}

pub async fn send_message(
    m: SendMessageMethod,
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
    let message = crate::services::database::messages::create_message(m.channel_id.clone(), client.get_user_id(), trimmed.to_owned()).await;
    for x in clients_locked.clone().iter_mut() {
        println!("Sending message to {}", x.get_user_id());
        let mut value_buffer = Vec::new();
        let value = RpcApiEvent {
            event: Event::NewMessage (NewMessageEvent {
                message: message.clone(),
                channel_id: m.channel_id.clone(),
            }),
        };
        value
            .serialize(
                &mut Serializer::new(&mut value_buffer)
                    .with_struct_map(),
            )
            .unwrap();
        println!("Serialized");
        let y = x.socket.clone();
        future::timeout(Duration::from_millis(5000), async move {
            y.lock().await.send(Message::Binary(value_buffer)).await
        }).await.unwrap_or_else(|_| Ok(())).unwrap_or_else(|e| println!("{:?}", e));
    }
    Response::SendMessage(SendMessageResponse { message_id: message.id })
}
