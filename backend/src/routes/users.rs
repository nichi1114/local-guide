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
    use crate::repository::auth::IdentityProfile;
    use crate::test_utils::router::{parse_json, TestContext};
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn returns_user_when_token_valid() {
        let ctx = TestContext::new(super::router).await;
        let repo = ctx.auth_repo();
        let user = repo
            .upsert_user_with_identity(IdentityProfile {
                provider: "google",
                provider_user_id: "user-123",
                email: Some("user@example.com"),
                name: Some("Test User"),
                avatar_url: None,
            })
            .await
            .expect("insert user");

        let token = ctx.jwt.generate(&user).expect("jwt");

        let response = ctx
            .app
            .oneshot(
                Request::get("/usr")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request succeeds");

        assert_eq!(response.status(), StatusCode::OK);

        let parsed: UserResponse = parse_json(response).await;
        assert_eq!(parsed.id, user.id);
        assert_eq!(parsed.email, user.email);
        assert_eq!(parsed.name, user.name);
    }

    #[tokio::test]
    async fn returns_unauthorized_without_header() {
        let ctx = TestContext::new(super::router).await;
        let response = ctx
            .app
            .oneshot(Request::get("/usr").body(Body::empty()).unwrap())
            .await
            .expect("request succeeds");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
