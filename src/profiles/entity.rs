use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Patient,
    Doctor,
    Admin,
}

#[derive(Debug, Clone, FromRow)]
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

    pub async fn get_id_by_auth_user_id(
        pool: &PgPool,
        auth_user_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM profiles WHERE auth_user_id = $1 AND deleted_at IS NULL",
        )
        .bind(auth_user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn upsert(
        pool: &PgPool,
        auth_user_id: Uuid,
        email: &str,
        role: UserRole,
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
        .bind(role)
        .fetch_one(pool)
        .await
    }

    pub async fn update_locale(
        pool: &PgPool,
        auth_user_id: Uuid,
        locale: String,
    ) -> Result<Option<Profile>, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles SET
                locale = $2,
                updated_at = NOW()
            WHERE auth_user_id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(auth_user_id)
        .bind(locale)
        .fetch_optional(pool)
        .await
    }

    pub async fn soft_delete(
        pool: &PgPool,
        auth_user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE profiles SET deleted_at = NOW()
            WHERE auth_user_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(auth_user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
