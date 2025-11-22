use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::repository::auth::UserRecord;
use crate::repository::place::{PlaceImageRecord, PlaceRecord};

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

#[derive(Serialize)]
pub struct PlaceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category: String,
    pub location: String,
    pub note: Option<String>,
    pub images: Vec<PlaceImageResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PlaceRecord> for PlaceResponse {
    fn from(value: PlaceRecord) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            name: value.name,
            category: value.category,
            location: value.location,
            note: value.note,
            images: Vec::new(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct PlaceImageResponse {
    pub id: Uuid,
    pub caption: Option<String>,
    pub download_url: String,
    pub created_at: DateTime<Utc>,
}

impl PlaceImageResponse {
    pub fn from_record(record: PlaceImageRecord) -> Self {
        let download_url = format!("/places/{}/images/{}", record.place_id, record.id);
        Self {
            id: record.id,
            caption: record.caption,
            download_url,
            created_at: record.created_at,
        }
    }
}
