use chrono::{DateTime, Utc};
use sqlx::{Error as SqlxError, FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PlaceRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] SqlxError),
}

type RepoResult<T> = Result<T, PlaceRepositoryError>;

#[derive(Clone)]
pub struct PlaceRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlaceRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category: String,
    pub location: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlaceImageRecord {
    pub id: Uuid,
    pub place_id: Uuid,
    pub file_name: String,
    pub caption: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewPlace<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: &'a str,
    pub category: &'a str,
    pub location: &'a str,
    pub note: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct NewPlaceImage<'a> {
    pub id: Uuid,
    pub place_id: Uuid,
    pub file_name: &'a str,
    pub caption: Option<&'a str>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePlace {
    pub name: Option<String>,
    pub category: Option<String>,
    pub location: Option<String>,
    pub note: Option<String>,
}

impl PlaceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_place_with_images(
        &self,
        payload: NewPlace<'_>,
        images: &[NewPlaceImage<'_>],
    ) -> RepoResult<(PlaceRecord, Vec<PlaceImageRecord>)> {
        let mut tx = self.pool.begin().await?;

        let place = sqlx::query_as::<_, PlaceRecord>(
            r#"
            INSERT INTO places (id, user_id, name, category, location, note)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, category, location, note, created_at, updated_at
            "#,
        )
        .bind(payload.id)
        .bind(payload.user_id)
        .bind(payload.name)
        .bind(payload.category)
        .bind(payload.location)
        .bind(payload.note)
        .fetch_one(tx.as_mut())
        .await?;

        let mut inserted_images = Vec::new();
        for img in images {
            let record = sqlx::query_as::<_, PlaceImageRecord>(
                r#"
                INSERT INTO place_images (id, place_id, file_name, caption)
                VALUES ($1, $2, $3, $4)
                RETURNING id, place_id, file_name, caption, created_at
                "#,
            )
            .bind(img.id)
            .bind(img.place_id)
            .bind(img.file_name)
            .bind(img.caption)
            .fetch_one(tx.as_mut())
            .await?;
            inserted_images.push(record);
        }

        tx.commit().await?;

        Ok((place, inserted_images))
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> RepoResult<Vec<PlaceRecord>> {
        let mut tx = self.pool.begin().await?;

        let records = sqlx::query_as::<_, PlaceRecord>(
            r#"
            SELECT id, user_id, name, category, location, note, created_at, updated_at
            FROM places
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(records)
    }

    pub async fn find_for_user(
        &self,
        user_id: Uuid,
        place_id: Uuid,
    ) -> RepoResult<Option<PlaceRecord>> {
        let mut tx = self.pool.begin().await?;

        let record = sqlx::query_as::<_, PlaceRecord>(
            r#"
            SELECT id, user_id, name, category, location, note, created_at, updated_at
            FROM places
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(place_id)
        .bind(user_id)
        .fetch_optional(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(record)
    }

    pub async fn update_place_with_images(
        &self,
        user_id: Uuid,
        place_id: Uuid,
        update: UpdatePlace,
        new_images: &[NewPlaceImage<'_>],
        delete_image_ids: &[Uuid],
    ) -> RepoResult<(PlaceRecord, Vec<PlaceImageRecord>, Vec<PlaceImageRecord>)> {
        let mut tx = self.pool.begin().await?;

        let place = sqlx::query_as::<_, PlaceRecord>(
            r#"
            UPDATE places
            SET name = COALESCE($3, name),
                category = COALESCE($4, category),
                location = COALESCE($5, location),
                note = COALESCE($6, note),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, category, location, note, created_at, updated_at
            "#,
        )
        .bind(place_id)
        .bind(user_id)
        .bind(update.name.as_deref())
        .bind(update.category.as_deref())
        .bind(update.location.as_deref())
        .bind(update.note.as_deref())
        .fetch_one(tx.as_mut())
        .await?;

        let mut deleted_images = Vec::new();
        if !delete_image_ids.is_empty() {
            deleted_images = sqlx::query_as::<_, PlaceImageRecord>(
                r#"
                DELETE FROM place_images
                WHERE id = ANY($1) AND place_id = $2
                RETURNING id, place_id, file_name, caption, created_at
                "#,
            )
            .bind(delete_image_ids)
            .bind(place_id)
            .fetch_all(tx.as_mut())
            .await?;
        }

        let mut inserted_images = Vec::new();
        for img in new_images {
            let record = sqlx::query_as::<_, PlaceImageRecord>(
                r#"
                INSERT INTO place_images (id, place_id, file_name, caption)
                VALUES ($1, $2, $3, $4)
                RETURNING id, place_id, file_name, caption, created_at
                "#,
            )
            .bind(img.id)
            .bind(img.place_id)
            .bind(img.file_name)
            .bind(img.caption)
            .fetch_one(tx.as_mut())
            .await?;
            inserted_images.push(record);
        }

        tx.commit().await?;

        Ok((place, inserted_images, deleted_images))
    }

    pub async fn list_images_for_place(
        &self,
        user_id: Uuid,
        place_id: Uuid,
    ) -> RepoResult<Vec<PlaceImageRecord>> {
        let mut tx = self.pool.begin().await?;

        let records = sqlx::query_as::<_, PlaceImageRecord>(
            r#"
            SELECT pi.id, pi.place_id, pi.file_name, pi.caption, pi.created_at
            FROM place_images pi
            JOIN places p ON p.id = pi.place_id
            WHERE pi.place_id = $1 AND p.user_id = $2
            ORDER BY pi.created_at DESC
            "#,
        )
        .bind(place_id)
        .bind(user_id)
        .fetch_all(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(records)
    }

    pub async fn find_image_for_user(
        &self,
        user_id: Uuid,
        image_id: Uuid,
    ) -> RepoResult<Option<PlaceImageRecord>> {
        let mut tx = self.pool.begin().await?;

        let record = sqlx::query_as::<_, PlaceImageRecord>(
            r#"
            SELECT pi.id, pi.place_id, pi.file_name, pi.caption, pi.created_at
            FROM place_images pi
            JOIN places p ON p.id = pi.place_id
            WHERE pi.id = $1 AND p.user_id = $2
            "#,
        )
        .bind(image_id)
        .bind(user_id)
        .fetch_optional(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(record)
    }

    pub async fn delete_place_for_user(
        &self,
        user_id: Uuid,
        place_id: Uuid,
    ) -> RepoResult<Option<(PlaceRecord, Vec<PlaceImageRecord>)>> {
        let mut tx = self.pool.begin().await?;

        let place = sqlx::query_as::<_, PlaceRecord>(
            r#"
            SELECT id, user_id, name, category, location, note, created_at, updated_at
            FROM places
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(place_id)
        .bind(user_id)
        .fetch_optional(tx.as_mut())
        .await?;

        let Some(place) = place else {
            tx.commit().await?;
            return Ok(None);
        };

        let images = sqlx::query_as::<_, PlaceImageRecord>(
            r#"
            SELECT id, place_id, file_name, caption, created_at
            FROM place_images
            WHERE place_id = $1
            "#,
        )
        .bind(place_id)
        .fetch_all(tx.as_mut())
        .await?;

        sqlx::query(
            r#"
            DELETE FROM places
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(place_id)
        .bind(user_id)
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(Some((place, images)))
    }
}
