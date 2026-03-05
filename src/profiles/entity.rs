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
    pub auth_user_id: Uuid,
    pub email: String,
    pub locale: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Profile {
    pub async fn find_by_auth_user_id(
        pool: &PgPool,
        auth_user_id: Uuid,
    ) -> Result<Option<Profile>, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE auth_user_id = $1 AND deleted_at IS NULL",
        )
        .bind(auth_user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, auth_user_id: Uuid) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles SET deleted_at = now()
            WHERE auth_user_id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(auth_user_id)
        .fetch_one(pool)
        .await
    }
}

// ── Create ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct SyncProfileInput {
    pub role: UserRole,
}

impl SyncProfileInput {
    pub async fn sync(
        pool: &PgPool,
        auth_user_id: Uuid,
        email: &str,
        data: SyncProfileInput,
    ) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            INSERT INTO profiles (auth_user_id, email, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (auth_user_id) DO UPDATE SET
                email = EXCLUDED.email,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(auth_user_id)
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
        auth_user_id: Uuid,
        data: UpdateProfileInput,
    ) -> Result<Profile, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles SET
                locale = COALESCE($2, locale),
                updated_at = NOW()
            WHERE auth_user_id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(auth_user_id)
        .bind(data.locale)
        .fetch_one(pool)
        .await
    }
}
