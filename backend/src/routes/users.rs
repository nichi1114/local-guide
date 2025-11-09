use axum::{
    extract::{Extension, State},
    http::StatusCode,
    middleware,
    routing::get,
    Json, Router,
};
use tracing::error;

use crate::app_state::AppState;
use crate::jwt::JwtClaims;

use super::middleware::jwt_auth;
use super::models::{ErrorResponse, UserResponse};

pub fn router(state: AppState) -> Router {
    let middleware_state = state.clone();
    Router::new()
        .route("/usr", get(current_user))
        .route_layer(middleware::from_fn_with_state(middleware_state, jwt_auth))
        .with_state(state)
}

async fn current_user(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.auth_repository();
    let user = repository
        .find_user_by_id(claims.sub)
        .await
        .map_err(|err| {
            error!(?err, "failed to fetch user");
            internal_error()
        })?
        .ok_or_else(user_not_found)?;

    Ok(Json(UserResponse::from(user)))
}

fn user_not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("user_not_found", "user does not exist")),
    )
}

fn internal_error() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            "internal_error",
            "unexpected server error",
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::db::create_pool;
    use crate::jwt::JwtManager;
    use crate::repository::auth::{AuthRepository, IdentityProfile, UserRecord};
    use crate::sql_init::run_initialization;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use tower::ServiceExt;

    const TEST_JWT_SECRET: &str = "secret";

    #[tokio::test]
    async fn returns_user_when_token_valid() {
        let pool = setup_pool().await;
        let repository = AuthRepository::new(pool);
        let (app, jwt, user) = build_app(repository).await;

        let token = jwt.generate(&user).expect("jwt");

        let response = app
            .oneshot(
                Request::get("/usr")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request succeeds");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let parsed: UserResponse = serde_json::from_slice(&body).expect("parse user");
        assert_eq!(parsed.id, user.id);
        assert_eq!(parsed.email, user.email);
        assert_eq!(parsed.name, user.name);
    }

    #[tokio::test]
    async fn returns_unauthorized_without_header() {
        let pool = setup_pool().await;
        let repository = AuthRepository::new(pool);
        let (app, _, _) = build_app(repository).await;

        let response = app
            .oneshot(Request::get("/usr").body(Body::empty()).unwrap())
            .await
            .expect("request succeeds");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    async fn build_app(repository: AuthRepository) -> (Router, JwtManager, UserRecord) {
        let jwt = JwtManager::new(TEST_JWT_SECRET.to_string(), 3600);

        let user = repository
            .upsert_user_with_identity(IdentityProfile {
                provider: "google",
                provider_user_id: "user-123",
                email: Some("user@example.com"),
                name: Some("Test User"),
                avatar_url: None,
            })
            .await
            .expect("insert user");

        let providers = HashMap::new();
        let app_state = AppState::new(providers, jwt.clone(), repository);
        (super::router(app_state), jwt, user)
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
}
