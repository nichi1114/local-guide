use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

const DEFAULT_MAX_CONNECTIONS: u32 = 20;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max_connections = env::var("MAX_DB_CONNECTIONS")
        .ok()
        .and_then(|val| val.parse::<u32>().ok())
        .unwrap_or(DEFAULT_MAX_CONNECTIONS);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    Ok(pool)
}
