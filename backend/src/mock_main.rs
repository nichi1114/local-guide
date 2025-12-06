#![allow(dead_code)]

use std::collections::HashMap;
use std::num::ParseIntError;
use std::path::PathBuf;

use axum::Router;
use thiserror::Error;
use tokio::net::TcpListener;

mod app_state;
mod auth_service;
mod db;
mod jwt;
mod oauth_config;
mod repository;
mod routes;
mod sql_init;
#[cfg(test)]
mod test_utils;

use app_state::AppState;
use auth_service::{AuthService, MockUserProfile};
use jwt::JwtManager;
use repository::auth::AuthRepository;
use repository::image_store::ImageStore;
use repository::place::PlaceRepository;
use sqlx::Error as SqlxError;

const DEFAULT_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/local_guide";
const DEFAULT_JWT_SECRET: &str = "mock-jwt-secret";
const DEFAULT_JWT_TTL_SECONDS: u64 = 3600;

#[tokio::main]
async fn main() -> Result<(), MockBackendError> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) => tracing::debug!("Could not load .env file: {}", e),
    }
    init_tracing();
    run().await
}

fn init_tracing() {
    use tracing_subscriber::{fmt, util::SubscriberInitExt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=warn"));

    fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_target(false)
        .finish()
        .init();
}

async fn run() -> Result<(), MockBackendError> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let pool = db::create_pool(&database_url).await?;

    if std::env::var("RUN_SQL_INIT")
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        sql_init::run_initialization(&pool).await?;
    }

    let repository = AuthRepository::new(pool.clone());
    let place_repository = PlaceRepository::new(pool.clone());

    let mut providers = HashMap::new();
    let mock_profile = resolve_mock_profile();
    providers.insert(
        "google".to_string(),
        AuthService::new_mock(repository.clone(), "google", mock_profile.clone()),
    );
    providers.insert(
        "google-ios".to_string(),
        AuthService::new_mock(repository.clone(), "google-ios", mock_profile),
    );

    let jwt_secret = resolve_jwt_secret();
    let jwt_ttl_seconds = match std::env::var("JWT_TTL_SECONDS") {
        Ok(value) => value
            .parse::<u64>()
            .map_err(MockBackendError::InvalidJwtTtl)?,
        Err(_) => DEFAULT_JWT_TTL_SECONDS,
    };
    let jwt_manager = JwtManager::new(jwt_secret, jwt_ttl_seconds);

    let place_image_dir = resolve_place_image_dir();
    let image_store = ImageStore::new(place_image_dir).map_err(MockBackendError::StartupIo)?;

    let state = AppState::new(
        providers,
        jwt_manager,
        repository,
        place_repository,
        image_store,
    );

    let app = build_router(state);

    let listen_addr = resolve_listen_addr();
    let listener = TcpListener::bind(&listen_addr).await?;
    let address = listener.local_addr()?;

    tracing::info!(%address, "mock backend listening for requests");

    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    routes::router(state)
}

fn resolve_listen_addr() -> String {
    if let Ok(addr) = std::env::var("BACKEND_BIND_ADDR") {
        let trimmed = addr.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    DEFAULT_ADDR.to_string()
}

fn resolve_place_image_dir() -> PathBuf {
    let configured =
        std::env::var("PLACE_IMAGE_DIR").unwrap_or_else(|_| "data/place_images".to_string());
    PathBuf::from(configured)
}

fn resolve_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .or_else(|_| std::env::var("MOCK_JWT_SECRET"))
        .unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string())
}

fn resolve_mock_profile() -> MockUserProfile {
    let default_email = "mock.user@example.com".to_string();
    let default_name = "Mock User".to_string();
    MockUserProfile {
        provider_user_id: std::env::var("MOCK_GOOGLE_USER_ID")
            .unwrap_or_else(|_| "mock-google-user".to_string()),
        email: Some(std::env::var("MOCK_GOOGLE_EMAIL").unwrap_or(default_email)),
        name: Some(std::env::var("MOCK_GOOGLE_NAME").unwrap_or(default_name)),
        avatar_url: std::env::var("MOCK_GOOGLE_AVATAR_URL").ok(),
    }
}

#[derive(Debug, Error)]
enum MockBackendError {
    #[error(transparent)]
    Sqlx(#[from] SqlxError),
    #[error("failed to initialize image directory: {0}")]
    StartupIo(#[from] std::io::Error),
    #[error(transparent)]
    Server(#[from] axum::Error),
    #[error("invalid JWT_TTL_SECONDS value: {0}")]
    InvalidJwtTtl(#[from] ParseIntError),
}
