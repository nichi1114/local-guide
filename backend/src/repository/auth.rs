use sqlx::{Error as SqlxError, FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] SqlxError),
}

type RepoResult<T> = Result<T, AuthRepositoryError>;

#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IdentityProfile<'a> {
    pub provider: &'a str,
    pub provider_user_id: &'a str,
    pub email: Option<&'a str>,
    pub name: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
}

impl AuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_user_with_identity(
        &self,
        payload: IdentityProfile<'_>,
    ) -> RepoResult<UserRecord> {
        let mut tx = self.pool.begin().await?;

        if let Some(existing) = self
            .find_user_by_identity_tx(&mut tx, payload.provider, payload.provider_user_id)
            .await?
        {
            let user = self
                .update_user_profile_tx(
                    &mut tx,
                    existing.id,
                    payload.email,
                    payload.name,
                    payload.avatar_url,
                )
                .await?;
            tx.commit().await?;
            return Ok(user);
        }

        let new_user_id = Uuid::new_v4();

        let user_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            WITH existing_identity AS (
                SELECT user_id
                FROM oauth_identities
                WHERE provider = $1 AND provider_user_id = $2
                FOR UPDATE
            ),
            inserted_user AS (
                INSERT INTO users (id, email, name, avatar_url)
                VALUES ($3, $4, $5, $6)
                ON CONFLICT (id) DO NOTHING
                RETURNING id
            ),
            resolved_user AS (
                SELECT COALESCE(
                    (SELECT user_id FROM existing_identity),
                    (SELECT id FROM inserted_user),
                    $3
                ) AS user_id
            ),
            identity_upsert AS (
                INSERT INTO oauth_identities (id, provider, provider_user_id, user_id)
                SELECT gen_random_uuid(), $1, $2, user_id FROM resolved_user
                ON CONFLICT (provider, provider_user_id)
                DO UPDATE SET user_id = EXCLUDED.user_id
                RETURNING user_id
            )
            SELECT user_id FROM identity_upsert
            "#,
        )
        .bind(payload.provider)
        .bind(payload.provider_user_id)
        .bind(new_user_id)
        .bind(payload.email)
        .bind(payload.name)
        .bind(payload.avatar_url)
        .fetch_one(tx.as_mut())
        .await?;

        if user_id != new_user_id {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(new_user_id)
                .execute(tx.as_mut())
                .await?;
        }

        let user = if user_id == new_user_id {
            UserRecord {
                id: user_id,
                email: payload.email.map(|s| s.to_string()),
                name: payload.name.map(|s| s.to_string()),
                avatar_url: payload.avatar_url.map(|s| s.to_string()),
            }
        } else {
            self.find_user_by_id(user_id)
                .await?
                .expect("user must exist after upsert")
        };

        tx.commit().await?;

        Ok(user)
    }

    async fn update_user_profile_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        email: Option<&str>,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> RepoResult<UserRecord> {
        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            UPDATE users
            SET email = COALESCE($2, email),
                name = COALESCE($3, name),
                avatar_url = COALESCE($4, avatar_url),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, name, avatar_url
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(name)
        .bind(avatar_url)
        .fetch_one(tx.as_mut())
        .await?;

        Ok(user)
    }

    async fn find_user_by_identity_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        provider: &str,
        provider_user_id: &str,
    ) -> RepoResult<Option<UserRecord>> {
        let record = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT u.id, u.email, u.name, u.avatar_url
            FROM oauth_identities oi
            JOIN users u ON u.id = oi.user_id
            WHERE oi.provider = $1 AND oi.provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(tx.as_mut())
        .await?;

        Ok(record)
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> RepoResult<Option<UserRecord>> {
        let record = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, email, name, avatar_url
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_init::run_initialization;
    use futures::future::join;
    use sqlx::PgPool;
    use tokio::task;

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/local_guide_test".into()
            });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect postgres");
        run_initialization(&pool).await.expect("apply schema");

        sqlx::query("TRUNCATE TABLE oauth_identities, users RESTART IDENTITY")
            .execute(&pool)
            .await
            .expect("truncate tables");

        pool
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_upserts_link_to_same_user() {
        let pool = setup_pool().await;
        let repo_a = AuthRepository::new(pool.clone());
        let repo_b = AuthRepository::new(pool.clone());

        let task1 = task::spawn({
            let repo = repo_a.clone();
            let profile = test_profile();
            async move { repo.upsert_user_with_identity(profile).await }
        });
        let task2 = task::spawn({
            let repo = repo_b.clone();
            let profile = test_profile();
            async move { repo.upsert_user_with_identity(profile).await }
        });

        let (res1, res2) = join(task1, task2).await;
        let user1 = res1.expect("task1 join").expect("task1 result");
        let user2 = res2.expect("task2 join").expect("task2 result");

        assert_eq!(
            user1.id, user2.id,
            "both operations should resolve to the same user"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1,
            "only one user row expected",
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM oauth_identities")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1,
            "only one identity row expected",
        );
    }

    fn test_profile() -> IdentityProfile<'static> {
        IdentityProfile {
            provider: "google",
            provider_user_id: "user-123",
            email: Some("user@example.com"),
            name: Some("Name"),
            avatar_url: Some("https://example.com/avatar.png"),
        }
    }
}
