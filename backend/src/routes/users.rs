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
        .route("/usr", get(current_user).delete(delete_user))
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

async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let auth_repository = state.auth_repository();
    let place_repository = state.place_repository();
    let image_store = state.image_store();

    let places = place_repository
        .list_for_user(claims.sub)
        .await
        .map_err(|err| {
            error!(?err, "failed to list places for user deletion");
            internal_error()
        })?;

    let mut files_by_place = Vec::new();
    for place in places {
        let images = place_repository
            .list_images_for_place(claims.sub, place.id)
            .await
            .map_err(|err| {
                error!(?err, "failed to list place images for user deletion");
                internal_error()
            })?;
        let file_names: Vec<String> = images.into_iter().map(|img| img.file_name).collect();
        files_by_place.push((place.id, file_names));
    }

    let deleted = auth_repository
        .delete_user(claims.sub)
        .await
        .map_err(|err| {
            error!(?err, "failed to delete user");
            internal_error()
        })?;

    if !deleted {
        return Err(user_not_found());
    }

    for (place_id, file_names) in files_by_place {
        image_store.remove_files(place_id, &file_names).await;
        image_store.remove_place_dir(place_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
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
    use crate::test_utils::router::{multipart_body, parse_json, Part, TestContext};
    use axum::body::Body;
    use axum::http::{header, Request};
    use tower::ServiceExt;
    use uuid::Uuid;

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

    #[tokio::test]
    async fn delete_user_removes_related_data_and_files() {
        // Use a router that includes both user and place routes so we can create and delete places.
        let ctx = TestContext::new(|state| {
            crate::routes::places::router(state.clone()).merge(super::router(state))
        })
        .await;
        let user = ctx.insert_user().await;
        let token = ctx.jwt.generate(&user).expect("jwt");

        let place_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        let (boundary, body) = multipart_body(vec![
            Part::text("id", place_id.to_string()),
            Part::text("name", "To Delete"),
            Part::text("category", "Cafe"),
            Part::text("location", "Somewhere"),
            Part::text("image_id", image_id.to_string()),
            Part::file("image", "img.jpg", "image/jpeg", vec![1, 2, 3]),
        ]);

        let create_response = ctx
            .app
            .clone()
            .oneshot(
                Request::post("/places")
                    .header("Authorization", format!("Bearer {}", token))
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("create place");
        assert_eq!(create_response.status(), StatusCode::OK);

        let file_name: String =
            sqlx::query_scalar("SELECT file_name FROM place_images WHERE id = $1")
                .bind(image_id)
                .fetch_one(&ctx.pool)
                .await
                .unwrap();
        let image_path = ctx.image_dir().join(place_id.to_string()).join(&file_name);
        assert!(image_path.exists());

        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::delete("/usr")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("delete user");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let user_row: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(&ctx.pool)
            .await
            .unwrap();
        assert!(user_row.is_none());

        let place_row: Option<Uuid> = sqlx::query_scalar("SELECT id FROM places WHERE id = $1")
            .bind(place_id)
            .fetch_optional(&ctx.pool)
            .await
            .unwrap();
        assert!(place_row.is_none());

        let image_row: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM place_images WHERE id = $1")
                .bind(image_id)
                .fetch_optional(&ctx.pool)
                .await
                .unwrap();
        assert!(image_row.is_none());

        assert!(!image_path.exists());
    }
}
