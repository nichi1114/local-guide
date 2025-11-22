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
use auth_service::{AuthService, AuthServiceBuildError};
use jwt::JwtManager;
use oauth_config::{OAuthConfigError, OAuthProviderConfig};
use repository::auth::AuthRepository;
use repository::image_store::ImageStore;
use repository::place::PlaceRepository;
use sqlx::Error as SqlxError;

const DEFAULT_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/local_guide";
const DEFAULT_JWT_TTL_SECONDS: u64 = 3600;

#[tokio::main]
async fn main() -> Result<(), BackendError> {
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

async fn run() -> Result<(), BackendError> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let pool = db::create_pool(&database_url).await?;
    // Only run SQL initialization if RUN_SQL_INIT is set to "true".
    // This is intended for development or CI environments only.
    if std::env::var("RUN_SQL_INIT")
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        sql_init::run_initialization(&pool).await?;
    }

    let repository = AuthRepository::new(pool.clone());
    let place_repository = PlaceRepository::new(pool.clone());
    let provider_configs = OAuthProviderConfig::load_from_env()?;

    let mut providers = HashMap::new();
    for config in provider_configs {
        let provider_id = config.provider_id.clone();
        let service = AuthService::new(repository.clone(), config)?;
        providers.insert(provider_id, service);
    }

    if providers.is_empty() {
        return Err(BackendError::NoProviders);
    }

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| BackendError::MissingJwtSecret)?;
    let jwt_ttl_seconds = match std::env::var("JWT_TTL_SECONDS") {
        Ok(value) => value.parse::<u64>().map_err(BackendError::InvalidJwtTtl)?,
        Err(_) => DEFAULT_JWT_TTL_SECONDS,
    };

    let jwt_manager = JwtManager::new(jwt_secret, jwt_ttl_seconds);

    let place_image_dir = resolve_place_image_dir();
    let image_store = ImageStore::new(place_image_dir).map_err(BackendError::StartupIo)?;

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

    tracing::info!(%address, "listening for requests");

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

#[derive(Debug, Error)]
enum BackendError {
    #[error(transparent)]
    Config(#[from] OAuthConfigError),
    #[error(transparent)]
    AuthInit(#[from] AuthServiceBuildError),
    #[error(transparent)]
    Sqlx(#[from] SqlxError),
    #[error("failed to initialize image directory: {0}")]
    StartupIo(#[from] std::io::Error),
    #[error(transparent)]
    Server(#[from] axum::Error),
    #[error("no OAuth providers configured")]
    NoProviders,
    #[error("JWT_SECRET environment variable must be set")]
    MissingJwtSecret,
    #[error("invalid JWT_TTL_SECONDS value: {0}")]
    InvalidJwtTtl(#[from] ParseIntError),
}
