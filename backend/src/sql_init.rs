use anyhow::Result;
use sqlx::PgPool;

pub async fn run_initialization(pool: &PgPool) -> Result<()> {
    let script = include_str!("../sql/init.sql");

    for statement in script.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }

        sqlx::query(trimmed).execute(pool).await?;
    }

    Ok(())
}
