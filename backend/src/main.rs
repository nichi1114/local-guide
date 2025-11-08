use std::collections::HashMap;

use anyhow::{bail, Context};
use axum::Router;
use tokio::net::TcpListener;

mod app_state;
mod auth_service;
mod db;
mod oauth_config;
mod repository;
mod routes;
mod sql_init;

use app_state::AppState;
use auth_service::AuthService;
use oauth_config::OAuthProviderConfig;
use repository::auth::AuthRepository;

const DEFAULT_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/local_guide";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

async fn run() -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let pool = db::create_pool(&database_url)
        .await
        .context("failed to create database pool")?;
    sql_init::run_initialization(&pool)
        .await
        .context("failed to run initialization SQL")?;

    let repository = AuthRepository::new(pool.clone());
    let provider_configs = OAuthProviderConfig::load_from_env()
        .context("failed to load OAuth provider configuration")?;

    let mut providers = HashMap::new();
    for config in provider_configs {
        let provider_id = config.provider_id.clone();
        let service = AuthService::new(repository.clone(), config)
            .with_context(|| format!("failed to initialize auth service for {provider_id}"))?;
        providers.insert(provider_id, service);
    }

    if providers.is_empty() {
        bail!("no OAuth providers configured");
    }

    let state = AppState::new(providers);

    let app = build_router(state);

    let listener = TcpListener::bind(DEFAULT_ADDR)
        .await
        .with_context(|| format!("failed to bind to {DEFAULT_ADDR}"))?;
    let address = listener
        .local_addr()
        .context("failed to read bound address")?;

    tracing::info!(%address, "listening for requests");

    axum::serve(listener, app).await.context("server failed")?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    routes::router(state)
}
