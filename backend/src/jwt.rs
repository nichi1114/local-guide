use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::repository::auth::UserRecord;

const BEARER: &str = "Bearer";

#[derive(Clone)]
pub struct JwtManager {
    secret: Arc<Vec<u8>>,
    expiration: Duration,
}

impl JwtManager {
    pub fn new(secret: String, expiration_seconds: u64) -> Self {
        Self {
            secret: Arc::new(secret.into_bytes()),
            expiration: Duration::seconds(expiration_seconds as i64),
        }
    }

    pub fn generate(&self, user: &UserRecord) -> Result<String, JwtError> {
        let issued_at = Utc::now();
        let claims = JwtClaims {
            sub: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            iat: issued_at.timestamp(),
            exp: (issued_at + self.expiration).timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key()).map_err(JwtError::EncodeFailed)
    }

    pub fn verify(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<JwtClaims>(token, &self.decoding_key(), &validation)
            .map(|data| data.claims)
            .map_err(JwtError::DecodeFailed)
    }

    fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(&self.secret)
    }

    fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(&self.secret)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("failed to encode JWT: {0}")]
    EncodeFailed(#[source] jsonwebtoken::errors::Error),
    #[error("failed to decode JWT: {0}")]
    DecodeFailed(#[source] jsonwebtoken::errors::Error),
}

pub fn split_bearer_token(header_value: &str) -> Option<&str> {
    let (schema, token) = header_value.split_once(' ')?;

    if schema.eq_ignore_ascii_case(BEARER) {
        Some(token)
    } else {
        None
    }
}
