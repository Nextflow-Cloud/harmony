use std::time::Duration;

use async_std::future;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::SinkExt;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, Result},
    services::{
        database::{messages::{Message}, channels::Channel},
        socket::RpcClient,
    },
};

use super::{Event, NewMessageEvent, Respond, Response, RpcApiEvent};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesMethod {
    channel_id: String,
    limit: Option<i64>,
    latest: Option<bool>,
    before: Option<String>,
    after: Option<String>,
}

#[async_trait]
impl Respond for GetMessagesMethod {
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        super::authentication::check_authenticated(&clients, &id)?;
        let channel = Channel::get(&self.channel_id).await?;
        let messages = channel.get_messages(
            self.limit,
            self.latest,
            self.before.clone(),
            self.after.clone(),
        )
        .await?;
        Ok(Response::GetMessages(GetMessagesResponse { messages }))
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
    async fn respond(&self, clients: DashMap<String, RpcClient>, id: String) -> Result<Response> {
        let user = super::authentication::check_authenticated(&clients, &id)?;
        let trimmed = self.content.trim();
        if trimmed.len() > 4096 {
            return Err(Error::MessageTooLong);
        }
        if trimmed.len() < 1 {
            return Err(Error::MessageEmpty);
        }
        let message = Message::create(
            self.channel_id.clone(),
            user.id.clone(),
            trimmed.to_owned(),
        )
        .await?;
        for x in clients.clone().iter_mut() {
            // TODO: Check if user is in channel
            if let Some(u) = &x.user {
                println!("Sending message to {}", u.id);
                let mut value_buffer = Vec::new();
                let value = RpcApiEvent {
                    event: Event::NewMessage(NewMessageEvent {
                        message: message.clone(),
                        channel_id: self.channel_id.clone(),
                    }),
                };
                value
                    .serialize(&mut Serializer::new(&mut value_buffer).with_struct_map())
                    .unwrap();
                println!("Serialized");
                let y = x.socket.clone();
                future::timeout(Duration::from_millis(5000), async move {
                    y.lock()
                        .await
                        .send(async_tungstenite::tungstenite::Message::Binary(
                            value_buffer,
                        ))
                        .await
                })
                .await
                .unwrap_or_else(|_| Ok(()))
                .unwrap_or_else(|e| println!("{:?}", e));
            }
        }
        Ok(Response::SendMessage(SendMessageResponse {
            message_id: message.id,
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    message_id: String,
}
