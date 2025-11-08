use anyhow::Result;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

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
    ) -> Result<UserRecord> {
        let mut tx = self.pool.begin().await?;

        if let Some(existing) = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT u.id, u.email, u.name, u.avatar_url
            FROM oauth_identities oi
            JOIN users u ON u.id = oi.user_id
            WHERE oi.provider = $1 AND oi.provider_user_id = $2
            "#,
        )
        .bind(payload.provider)
        .bind(payload.provider_user_id)
        .fetch_optional(tx.as_mut())
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

        let user = self
            .insert_user_tx(&mut tx, payload.email, payload.name, payload.avatar_url)
            .await?;

        self.insert_identity_tx(&mut tx, payload.provider, payload.provider_user_id, user.id)
            .await?;

        tx.commit().await?;
        Ok(user)
    }

    async fn insert_user_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        email: Option<&str>,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<UserRecord> {
        let id = Uuid::new_v4();
        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (id, email, name, avatar_url)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, name, avatar_url
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(name)
        .bind(avatar_url)
        .fetch_one(tx.as_mut())
        .await?;

        Ok(user)
    }

    async fn update_user_profile_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        email: Option<&str>,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<UserRecord> {
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

    async fn insert_identity_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        provider: &str,
        provider_user_id: &str,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO oauth_identities (id, provider, provider_user_id, user_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (provider, provider_user_id) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(provider)
        .bind(provider_user_id)
        .bind(user_id)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }
}
