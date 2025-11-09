use serde::Serialize;
use uuid::Uuid;

use crate::repository::auth::UserRecord;

#[cfg_attr(test, derive(serde::Deserialize))]
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<UserRecord> for UserResponse {
    fn from(value: UserRecord) -> Self {
        Self {
            id: value.id,
            email: value.email,
            name: value.name,
            avatar_url: value.avatar_url,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            error,
            message: message.into(),
        }
    }
}
