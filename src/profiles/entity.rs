use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Patient,
    Doctor,
    Admin,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Profile {
    pub id: Uuid,
    pub auth_uid: Uuid,
    pub email: String,
    pub locale: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Profile {
    /// List all active profiles (admin)
    pub async fn list(pool: &PgPool, include_deleted: bool) -> Result<Vec<Profile>, sqlx::Error> {
        let query = if include_deleted {
            "SELECT * FROM profiles ORDER BY created_at DESC"
        } else {
            "SELECT * FROM profiles WHERE deleted_at IS NULL ORDER BY created_at DESC"
        };
        sqlx::query_as::<_, Profile>(query).fetch_all(pool).await
    }

    /// Get profile by id
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Get profile by Supabase auth_uid
    pub async fn find_by_auth_uid(
        pool: &PgPool,
        auth_uid: Uuid,
    ) -> Result<Option<Profile>, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE auth_uid = $1 AND deleted_at IS NULL",
        )
        .bind(auth_uid)
        .fetch_optional(pool)
        .await
    }

    /// Soft delete (RGPD — garde la trace)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles SET deleted_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Hard delete (RGPD — suppression définitive)
    pub async fn destroy(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM profiles WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Create ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct SyncProfileInput {
    pub role: UserRole,
}

impl SyncProfileInput {
    /// Upsert profile depuis Supabase Auth (appelé au premier login)
    pub async fn sync(
        pool: &PgPool,
        auth_uid: Uuid,
        email: &str,
        data: SyncProfileInput,
    ) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            INSERT INTO profiles (auth_uid, email, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (auth_uid) DO UPDATE SET
                email = EXCLUDED.email,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(auth_uid)
        .bind(email)
        .bind(data.role)
        .fetch_one(pool)
        .await
    }
}

// ── Update ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateProfileInput {
    pub locale: Option<String>,
}

impl UpdateProfileInput {
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: UpdateProfileInput,
    ) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles SET
                locale = COALESCE($2, locale),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(data.locale)
        .fetch_one(pool)
        .await
    }
}
