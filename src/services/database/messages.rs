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
}

pub async fn get_messages(channel_id: String) -> Vec<Message> {
    let database = super::get_database();
    let options = mongodb::options::FindOptions::builder()
        .sort(doc! {
            "created_at": -1,
        })
        .limit(100)
        .build();
    let cursor = database
        .collection::<Message>(&format!("messages_{}", channel_id))
        .find(
            doc! {
                "id": channel_id,
            },
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
    };
    let database = super::get_database();
    database
        .collection::<Message>(&format!("messages_{}", channel_id))
        .insert_one(message.clone(), None)
        .await
        .unwrap();
    message
}
