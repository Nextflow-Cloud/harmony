use mongodb::{bson::doc, error::Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Scope {
    id: String,
    name: String,
    disabled: bool,
    nexuses: Vec<String>,
    channels: Vec<String>,
    users: Vec<String>,
}

// Must be platform_administrator
pub async fn create_scope(name: String) -> Result<(), Error> {
    let scopes = super::get_database().collection::<Scope>("scopes");
    scopes
        .insert_one(
            Scope {
                id: crate::services::encryption::generate_id(),
                name,
                disabled: false,
                nexuses: Vec::new(),
                channels: Vec::new(),
                users: Vec::new(),
            },
            None,
        )
        .await?;
    Ok(())
}

pub async fn update_scope(
    id: String,
    name: Option<String>,
    disabled: Option<bool>,
    add_users: Vec<String>,
    remove_users: Vec<String>,
) -> Result<bool, Error> {
    let scopes = super::get_database().collection::<Scope>("scopes");
    let scope = scopes
        .find_one(
            doc! {
                "id": id
            },
            None,
        )
        .await?;
    match scope {
        Some(mut s) => {
            if let Some(n) = name {
                s.name = n;
            }
            if let Some(d) = disabled {
                s.disabled = d;
            }
            for ru in remove_users {
                let index = s.users.iter().position(|x| *x == ru);
                if let Some(i) = index {
                    s.users.remove(i);
                }
            }
            for au in add_users {
                let user = super::users::get_user(au.clone()).await;
                if user.is_ok() && user.unwrap().is_some() {
                    s.users.push(au);
                }
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

// TODO: warn user that this is a destructive action
pub async fn delete_scope(id: String) -> Result<bool, Error> {
    if id == "global" {
        Ok(false) // The global scope may not be deleted
    } else {
        let scopes = super::get_database().collection::<Scope>("scopes");
        scopes
            .delete_one(
                doc! {
                    "id": id
                },
                None,
            )
            .await?;
        Ok(true)
    }
}
