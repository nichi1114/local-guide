use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

const DEFAULT_MAX_CONNECTIONS: u32 = 5;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .connect(database_url)
        .await?;

    Ok(pool)
}
