use core::slice::SlicePattern;

use rmp_serde::{decode::Error, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, hyper::body::Bytes, Reply};

use crate::services::database::users::Presence;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RpcApiMethod {
    CreateScope {
        name: String,
    },
    DeleteScope {
        id: String,
    },

    FetchUser {
        id: Option<String>,
    },
    UpdateUser {
        profile_banner: Option<String>, // TODO: Make use of file handling
        profile_description: Option<String>,
        presence: Option<Presence>,
    },
    // BEGIN global scope only
    AddFriend {
        user_id: String,
    },
    RemoveFriend {
        user_id: String,
    },
    LookupUser {
        username: String,
    },
    // lookup user by id?
    // returns PartialUser
    FetchMutual {
        user_id: String,
    },
    AddBlock {
        user_id: String,
    },
    RemoveBlock {
        user_id: String,
    },
    // END global scope only
    FetchPrivateChannels {},

    CreateChannel {
        // TODO: ChannelType in database
    },
    FetchChannel {
        id: String,
    },
    UpdateChannel {},
    DeleteChannel {
        id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RpcApiResponse {
    Error { error: String },
}

// Warp router
pub fn routes(data: Bytes) -> impl Reply {
    let mut deserializer = Deserializer::new(data.as_slice());
    let result: Result<RpcApiMethod, Error> = Deserialize::deserialize(&mut deserializer);
    match result {
        Ok(_) => todo!(),
        Err(_) => {
            let mut buf = Vec::new();
            let val = RpcApiResponse::Error {
                error: "Invalid encoding".to_string(),
            };
            val.serialize(&mut Serializer::new(&mut buf)).unwrap();
            warp::reply::with_status(buf, StatusCode::BAD_REQUEST)
        }
    }
}
