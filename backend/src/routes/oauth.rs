use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_service::{AuthError, AuthSession};
use crate::repository::auth::UserRecord;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/:provider/exchange", post(exchange_code))
        .with_state(state)
}

#[derive(Deserialize)]
struct ProviderPath {
    provider: String,
}

#[derive(Deserialize)]
struct ExchangeRequest {
    code: String,
    code_verifier: String,
}

#[derive(Serialize)]
struct LoginResponse {
    user: UserResponse,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Serialize)]
struct UserResponse {
    id: Uuid,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

async fn exchange_code(
    State(state): State<AppState>,
    Path(path): Path<ProviderPath>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let Some(service) = state.auth_service(&path.provider) else {
        return Err(provider_not_configured(&path.provider));
    };

    let session = service
        .exchange_code(&payload.code, &payload.code_verifier)
        .await
        .map_err(map_auth_error)?;

    Ok(Json(LoginResponse::from(session)))
}

fn provider_not_configured(provider: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "provider_not_configured",
            message: format!("provider '{provider}' is not configured"),
        }),
    )
}

fn map_auth_error(error: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    error!(?error, "OAuth login failed");
    match error {
        AuthError::TokenExchange(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "token_exchange_failed",
                message: "failed to exchange authorization code with provider".to_string(),
            }),
        ),
        AuthError::UserInfo(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "userinfo_failed",
                message: "failed to fetch user information from provider".to_string(),
            }),
        ),
        AuthError::Storage(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "storage_error",
                message: "unexpected storage error".to_string(),
            }),
        ),
    }
}

impl From<AuthSession> for LoginResponse {
    fn from(value: AuthSession) -> Self {
        Self {
            user: UserResponse::from(value.user.clone()),
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expires_in: value.expires_in,
        }
    }
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
