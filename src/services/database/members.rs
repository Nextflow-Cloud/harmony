use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub space_id: String,
    pub roles: Vec<String>,
}

pub async fn delete_member(member_id: String, space_id: String) -> Result<()> {
    let database = super::get_database();
    database
        .collection::<Member>("members")
        .delete_one(doc! { 
            "id": member_id,
            "space_id": space_id    
        }, None)
        .await?;
    Ok(())
}

pub async fn get_member(member_id: String, space_id: String) -> Result<Member> {
    let database = super::get_database();
    let member = database
        .collection::<Member>("members")
        .find_one(doc! { 
            "id": member_id,
            "space_id": space_id,
        }, None)
        .await?
        .ok_or_else(|| crate::errors::Error::NotFound)?;
    Ok(member)
}
