use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;

mod routes;

const DEFAULT_ADDR: &str = "0.0.0.0:8080";

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
    let app = build_router();

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

fn build_router() -> Router {
    routes::router()
}
