use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::app_state::AppState;
use crate::auth_service::AuthError;
use crate::jwt::JwtError;
use crate::repository::auth::UserRecord;

use super::models::{ErrorResponse, UserResponse};

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
    code_verifier: Option<String>,
}

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize)]
struct LoginResponse {
    user: UserResponse,
    jwt_token: String,
}

async fn complete_callback(
    State(state): State<AppState>,
    Path(path): Path<ProviderPath>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    finish_login(
        state,
        path.provider,
        payload.code,
        payload.code_verifier.as_deref(),
    )
    .await
}

async fn finish_login(
    state: AppState,
    provider: String,
    code: String,
    code_verifier: Option<&str>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let Some(service) = state.auth_service(&provider) else {
        return Err(provider_not_configured(&provider));
    };
    let jwt_manager = state.jwt_manager();

    let user = service
        .complete_oauth_flow(&code, code_verifier)
        .await
        .map_err(map_auth_error)?;

    let jwt_token = jwt_manager.generate(&user).map_err(map_jwt_error)?;

    Ok(Json(LoginResponse::from_user(user, jwt_token)))
}

fn provider_not_configured(provider: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new(
            "provider_not_configured",
            format!("provider '{provider}' is not configured"),
        )),
    )
}

fn map_auth_error(error: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    error!(?error, "OAuth login failed");
    match error {
        AuthError::TokenExchange(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::new(
                "token_exchange_failed",
                "failed to exchange authorization code with provider",
            )),
        ),
        AuthError::UserInfo(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::new(
                "userinfo_failed",
                "failed to fetch user information from provider",
            )),
        ),
        AuthError::Storage(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "storage_error",
                "unexpected storage error",
            )),
        ),
    }
}

fn map_jwt_error(error: JwtError) -> (StatusCode, Json<ErrorResponse>) {
    error!(?error, "failed to generate JWT");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            "jwt_error",
            "failed to generate JWT token",
        )),
    )
}

impl LoginResponse {
    fn from_user(user: UserRecord, jwt_token: String) -> Self {
        Self {
            user: UserResponse::from(user),
            jwt_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::auth_service::AuthService;
    use crate::db::create_pool;
    use crate::jwt::JwtManager;
    use crate::oauth_config::OAuthProviderConfig;
    use crate::repository::auth::AuthRepository;
    use crate::repository::image_store::ImageStore;
    use crate::repository::place::PlaceRepository;
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

    const TEST_JWT_SECRET: &str = "jwt-test-secret";

    // NOTE: We intentionally hand-roll the test context here instead of using
    // `test_utils::router::TestContext` because these tests must inject a mock
    // OAuth provider (via Wiremock) and build a custom router per provider.
    // Reusing the shared helper would hide those knobs.
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
        assert!(
            !parsed.jwt_token.is_empty(),
            "jwt token should be present in response"
        );

        let jwt = JwtManager::new(TEST_JWT_SECRET.to_string(), 3600);
        let claims = jwt.verify(&parsed.jwt_token).expect("valid jwt");
        assert_eq!(claims.email.as_deref(), Some("user@example.com"));
        assert_eq!(claims.name.as_deref(), Some("Test User"));

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

        sqlx::query(
            "TRUNCATE TABLE place_images, places, oauth_identities, users RESTART IDENTITY",
        )
        .execute(&pool)
        .await
        .expect("truncate tables");

        pool
    }

    fn build_state(mock_server: &MockServer, pool: PgPool) -> AppState {
        let repository = AuthRepository::new(pool.clone());
        let place_repository = PlaceRepository::new(pool);
        let config = OAuthProviderConfig {
            provider_id: "google".to_string(),
            client_id: "client-id".to_string(),
            auth_url: format!("{}/auth", mock_server.uri()),
            token_url: format!("{}/token", mock_server.uri()),
            userinfo_url: format!("{}/userinfo", mock_server.uri()),
            redirect_uri: "https://example.com/callback".to_string(),
        };

        let service = AuthService::new(repository.clone(), config)
            .expect("initialize auth service for tests");

        let mut providers = HashMap::new();
        providers.insert("google".to_string(), service);
        AppState::new(
            providers,
            JwtManager::new(TEST_JWT_SECRET.to_string(), 3600),
            repository,
            place_repository,
            ImageStore::new(temp_image_dir()).expect("image store"),
        )
    }

    fn temp_image_dir() -> std::path::PathBuf {
        let path = std::env::temp_dir().join("local-guide-backend-tests");
        std::fs::create_dir_all(&path).expect("create temp image dir");
        path
    }
}
