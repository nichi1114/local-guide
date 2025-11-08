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
        .route("/auth/:provider/callback", post(complete_callback))
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

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize)]
struct LoginResponse {
    user: UserResponse,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[cfg_attr(test, derive(Deserialize))]
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

async fn complete_callback(
    State(state): State<AppState>,
    Path(path): Path<ProviderPath>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let Some(service) = state.auth_service(&path.provider) else {
        return Err(provider_not_configured(&path.provider));
    };

    let session = service
        .complete_oauth_flow(&payload.code, &payload.code_verifier)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::auth_service::AuthService;
    use crate::db::create_pool;
    use crate::oauth_config::OAuthProviderConfig;
    use crate::repository::auth::AuthRepository;
    use crate::sql_init::run_initialization;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use serde_json::json;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use tower::ServiceExt;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn callback_creates_user_and_returns_profile() {
        let pool = setup_pool().await;
        let mock_server = MockServer::start().await;
        let app = super::router(build_state(&mock_server, pool.clone()));

        let access_token_response = json!({
            "access_token": "mock-access-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "mock-refresh-token"
        });

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(access_token_response))
            .mount(&mock_server)
            .await;

        let user_info_response = json!({
            "sub": "google-user-123",
            "email": "user@example.com",
            "name": "Test User",
            "picture": "https://example.com/avatar.png"
        });

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .and(header("authorization", "Bearer mock-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user_info_response))
            .mount(&mock_server)
            .await;

        let payload = json!({
            "code": "auth-code",
            "code_verifier": "verifier"
        });

        let response = app
            .oneshot(
                Request::post("/auth/google/callback")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("request succeeds");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let parsed: LoginResponse = serde_json::from_slice(&body).expect("valid response");

        assert_eq!(parsed.user.email.as_deref(), Some("user@example.com"));
        assert_eq!(parsed.user.name.as_deref(), Some("Test User"));
        assert_eq!(parsed.refresh_token.as_deref(), Some("mock-refresh-token"));

        let stored_email: Option<String> = sqlx::query_scalar("SELECT email FROM users LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("stored user");
        assert_eq!(stored_email.as_deref(), Some("user@example.com"));

        let identities: String =
            sqlx::query_scalar("SELECT provider_user_id FROM oauth_identities WHERE provider = $1")
                .bind("google")
                .fetch_one(&pool)
                .await
                .expect("stored identity");
        assert_eq!(identities, "google-user-123");
    }

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/local_guide_test".into()
            });

        let pool = create_pool(&database_url)
            .await
            .expect("connect to postgres");
        run_initialization(&pool).await.expect("apply schema");

        sqlx::query("TRUNCATE TABLE oauth_identities, users RESTART IDENTITY")
            .execute(&pool)
            .await
            .expect("truncate tables");

        pool
    }

    fn build_state(mock_server: &MockServer, pool: PgPool) -> AppState {
        let repository = AuthRepository::new(pool);
        let config = OAuthProviderConfig {
            provider_id: "google".to_string(),
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            auth_url: format!("{}/auth", mock_server.uri()),
            token_url: format!("{}/token", mock_server.uri()),
            userinfo_url: format!("{}/userinfo", mock_server.uri()),
            redirect_uri: "https://example.com/callback".to_string(),
        };

        let service =
            AuthService::new(repository, config).expect("initialize auth service for tests");

        let mut providers = HashMap::new();
        providers.insert("google".to_string(), service);
        AppState::new(providers)
    }
}
