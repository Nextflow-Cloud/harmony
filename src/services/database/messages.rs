use async_std::stream::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) author_id: String,
    pub(crate) created_at: i64,
    pub(crate) edited: bool,
    pub(crate) edited_at: Option<i64>,
    pub(crate) channel_id: String,
}

pub async fn get_messages(
    channel_id: String,
    limit: Option<i64>,
    latest: Option<bool>,
    before: Option<String>,
    after: Option<String>,
) -> Vec<Message> {
    let database = super::get_database();
    let limit = limit.unwrap_or(50);
    let mut query = doc! { "channelId": channel_id };
    if let Some(before) = before {
        query.insert("id", doc! { "$lt": before });
    }
    if let Some(after) = after {
        query.insert("id", doc! { "$gt": after });
    }
    let options = mongodb::options::FindOptions::builder()
        .sort(doc! {
            "id": if latest.unwrap_or(false) { -1 } else { 1 }
        })
        .limit(limit)
        .build();
    let cursor = database
        .collection::<Message>("messages")
        .find(query, options)
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
        created_at: chrono::Utc::now().timestamp_millis(),
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

pub async fn edit_message(message_id: String, content: String) -> Option<Message> {
    let database = super::get_database();
    let message = database
        .collection::<Message>("messages")
        .find_one_and_update(
            doc! { "id": message_id },
            doc! { "$set": {
                "content": content,
                "edited": true,
                "editedAt": chrono::Utc::now().timestamp_millis(),
            } },
            None,
        )
        .await
        .unwrap();
    message
}

pub async fn delete_message(message_id: String) -> Option<Message> {
    let database = super::get_database();
    let message = database
        .collection::<Message>("messages")
        .find_one_and_delete(doc! { "id": message_id }, None)
        .await
        .unwrap();
    message
}
