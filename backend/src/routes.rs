use axum::Router;

use crate::app_state::AppState;

mod middleware;
mod models;
mod oauth;
mod users;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(oauth::router(state.clone()))
        .merge(users::router(state))
}
