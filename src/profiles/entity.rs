use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub supabase_uid: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub async fn find_by_supabase_uid(
        pool: &PgPool,
        supabase_uid: Uuid,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE supabase_uid = $1")
            .bind(supabase_uid)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_id_by_supabase_uid(
        pool: &PgPool,
        supabase_uid: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE supabase_uid = $1")
            .bind(supabase_uid)
            .fetch_optional(pool)
            .await
    }

    pub async fn upsert(
        pool: &PgPool,
        supabase_uid: Uuid,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (supabase_uid, email)
            VALUES ($1, $2)
            ON CONFLICT (supabase_uid) DO UPDATE SET
                email = EXCLUDED.email,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(supabase_uid)
        .bind(email)
        .fetch_one(pool)
        .await
    }

    pub async fn update_profile(
        pool: &PgPool,
        supabase_uid: Uuid,
        username: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET
                username = COALESCE($2, username),
                avatar_url = COALESCE($3, avatar_url),
                updated_at = NOW()
            WHERE supabase_uid = $1
            RETURNING *
            "#,
        )
        .bind(supabase_uid)
        .bind(username)
        .bind(avatar_url)
        .fetch_optional(pool)
        .await
    }
}
