use anyhow::Result;
use sqlx::PgPool;

pub async fn run_initialization(pool: &PgPool) -> Result<()> {
    let script = include_str!("../sql/init.sql");

    // Execute the entire script at once to avoid splitting on semicolons inside strings.
    sqlx::query(script).execute(pool).await?;
    Ok(())
}
