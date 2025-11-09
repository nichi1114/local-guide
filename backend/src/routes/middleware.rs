use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use tracing::error;

use crate::{app_state::AppState, jwt::split_bearer_token};

use super::models::ErrorResponse;

pub async fn jwt_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(req.headers())
        .ok_or_else(|| unauthorized("missing Authorization bearer token"))?;

    let claims = state
        .jwt_manager()
        .verify(token)
        .map_err(|error| {
            error!(?error, "failed to verify JWT");
            unauthorized("invalid or expired token")
        })?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

fn extract_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(split_bearer_token)
}

fn unauthorized(message: &'static str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse::new("unauthorized", message)),
    )
}
