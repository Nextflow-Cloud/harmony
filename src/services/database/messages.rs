use async_std::stream::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) author_id: String,
    pub(crate) created_at: u64,
    pub(crate) edited: bool,
    pub(crate) edited_at: Option<u64>,
    pub(crate) channel_id: String,
}

pub async fn get_messages(channel_id: String, limit: Option<i64>, oldest: Option<bool>, before: Option<String>, after: Option<String>) -> Vec<Message> {
    let database = super::get_database();
    let limit = limit.unwrap_or(50);
    let mut query = doc! { "channel_id": channel_id };
    if let Some(before) = before {
        query.insert("id", doc! { "$lt": before });
    }
    if let Some(after) = after {
        query.insert("id", doc! { "$gt": after });
    }
    let options = mongodb::options::FindOptions::builder()
        .sort(doc! {
            "id": if oldest.unwrap_or(false) { -1 } else { 1 }
        })
        .limit(limit)
        .build();
    let cursor = database
        .collection::<Message>("messages")
        .find(
            query,
            options,
        )
        .await;
    let mut messages = Vec::new();
    if let Ok(mut cursor) = cursor {
        while let Some(message) = cursor.next().await {
            messages.push(message.unwrap());
        }
    }
    messages
}

pub async fn create_message(channel_id: String, author_id: String, content: String) -> Message {
    let message = Message {
        id: Ulid::new().to_string(),
        content,
        author_id,
        created_at: chrono::Utc::now().timestamp_millis() as u64,
        edited: false,
        edited_at: None,
        channel_id,
    };
    let database = super::get_database();
    database
        .collection::<Message>("messages")
        .insert_one(message.clone(), None)
        .await
        .unwrap();
    message
}
