use axum::Router;

use crate::app_state::AppState;

mod oauth;

pub fn router(state: AppState) -> Router {
    oauth::router(state)
}
