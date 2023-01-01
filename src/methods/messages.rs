use std::time::Duration;

use async_std::future;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::SinkExt;
use rmp_serde::Serializer;
use serde::{Serialize, Deserialize};

use crate::{services::{socket::RpcClient, database::messages::{Message, get_messages, create_message}}, errors::Error};

use super::{Response, ErrorResponse, NewMessageEvent, Event, RpcApiEvent, Respond};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesMethod {
    channel_id: String,
    limit: Option<i64>, 
    latest: Option<bool>, 
    before: Option<String>, 
    after: Option<String>
}

#[async_trait]
impl Respond for GetMessagesMethod {
    async fn respond(&self, _: DashMap<String, RpcClient>, _: String) -> Response {
        let messages = get_messages(self.channel_id.clone(), self.limit, self.latest, self.before.clone(), self.after.clone()).await;
        Response::GetMessages(GetMessagesResponse { messages })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    messages: Vec<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageMethod {
    channel_id: String,
    content: String,
}

#[async_trait]
impl Respond for SendMessageMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Response {
        let trimmed = self.content.trim();
        if trimmed.len() > 4096 {
            return Response::Error(ErrorResponse {
                error: Error::MessageTooLong,
            });
        }
        if trimmed.len() < 1 {
            return Response::Error(ErrorResponse {
                error: Error::MessageEmpty,
            });
        }
        let client = clients.get(&id).unwrap();
        let message = create_message(self.channel_id.clone(), client.get_user_id(), trimmed.to_owned()).await;
        for x in clients.clone().iter_mut() {
            println!("Sending message to {}", x.get_user_id());
            let mut value_buffer = Vec::new();
            let value = RpcApiEvent {
                event: Event::NewMessage (NewMessageEvent {
                    message: message.clone(),
                    channel_id: self.channel_id.clone(),
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
                y.lock().await.send(async_tungstenite::tungstenite::Message::Binary(value_buffer)).await
            }).await.unwrap_or_else(|_| Ok(())).unwrap_or_else(|e| println!("{:?}", e));
        }
        Response::SendMessage(SendMessageResponse { message_id: message.id })
    }
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    message_id: String,
}
