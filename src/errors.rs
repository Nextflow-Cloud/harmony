use std::fmt;

use serde::{Deserialize, Serialize};

use crate::services::permissions::Permission;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "error", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
    // Generic errors
    DatabaseError { message: String },
    NotFound,
    Unimplemented,
    InvalidMethod,
    InvalidRequestId,
    InternalError,
    MissingPermission { permission: Permission },

    // Authentication errors
    InvalidToken,
    NotAuthenticated,

    // Message errors
    MessageTooLong,
    MessageEmpty,

    // Space errors
    NameTooLong,
    NameEmpty,

    // Invite errors
    InvalidInvite,
    InviteExpired,
    InviteAlreadyUsed,

    // Channel errors
    ChannelFull,

    // User errors
    Blocked,
    AlreadyFriends,
    AlreadyRequested,
    NotFriends,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::error::Error for Error {}

impl From<mongodb::error::Error> for Error {
    fn from(error: mongodb::error::Error) -> Self {
        Error::DatabaseError {
            message: error.to_string(),
        }
    }
}

impl From<mongodb::bson::ser::Error> for Error {
    fn from(error: mongodb::bson::ser::Error) -> Self {
        Error::DatabaseError {
            message: error.to_string(),
        }
    }
}
