use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Doctor {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub specialty: String,
    pub license_number: Option<String>,
    pub hospital_name: Option<String>,
    pub phone_professional: Option<String>,
    pub timezone: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Doctor {
    /// Liste tous les médecins
    pub async fn list(pool: &PgPool) -> Result<Vec<Doctor>, sqlx::Error> {
        sqlx::query_as::<_, Doctor>(
            "SELECT * FROM doctors ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// Liste les médecins vérifiés uniquement
    pub async fn list_verified(pool: &PgPool) -> Result<Vec<Doctor>, sqlx::Error> {
        sqlx::query_as::<_, Doctor>(
            "SELECT * FROM doctors WHERE is_verified = true ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// Récupère un médecin par id
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Doctor, sqlx::Error> {
        sqlx::query_as::<_, Doctor>("SELECT * FROM doctors WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Récupère le médecin lié à un profil
    pub async fn find_by_profile_id(
        pool: &PgPool,
        profile_id: Uuid,
    ) -> Result<Option<Doctor>, sqlx::Error> {
        sqlx::query_as::<_, Doctor>("SELECT * FROM doctors WHERE profile_id = $1")
            .bind(profile_id)
            .fetch_optional(pool)
            .await
    }

    /// Vérifie un médecin (admin)
    pub async fn verify(pool: &PgPool, id: Uuid) -> Result<Doctor, sqlx::Error> {
        sqlx::query_as::<_, Doctor>(
            r#"
            UPDATE doctors SET is_verified = true, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Supprime un médecin (hard delete)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM doctors WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Create ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateDoctorInput {
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub hospital_name: Option<String>,
    pub phone_professional: Option<String>,
    pub timezone: Option<String>,
}

impl CreateDoctorInput {
    pub async fn create(
        pool: &PgPool,
        profile_id: Uuid,
        data: CreateDoctorInput,
    ) -> Result<Doctor, sqlx::Error> {
        sqlx::query_as::<_, Doctor>(
            r#"
            INSERT INTO doctors (profile_id, specialty, license_number, hospital_name, phone_professional, timezone)
            VALUES ($1, COALESCE($2, 'chirurgie_bariatrique'), $3, $4, $5, COALESCE($6, 'Europe/Paris'))
            RETURNING *
            "#,
        )
        .bind(profile_id)
        .bind(data.specialty)
        .bind(data.license_number)
        .bind(data.hospital_name)
        .bind(data.phone_professional)
        .bind(data.timezone)
        .fetch_one(pool)
        .await
    }
}

// ── Update ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateDoctorInput {
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub hospital_name: Option<String>,
    pub phone_professional: Option<String>,
    pub timezone: Option<String>,
}

impl UpdateDoctorInput {
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: UpdateDoctorInput,
    ) -> Result<Doctor, sqlx::Error> {
        sqlx::query_as::<_, Doctor>(
            r#"
            UPDATE doctors SET
                specialty           = COALESCE($2, specialty),
                license_number      = COALESCE($3, license_number),
                hospital_name       = COALESCE($4, hospital_name),
                phone_professional  = COALESCE($5, phone_professional),
                timezone            = COALESCE($6, timezone),
                updated_at          = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(data.specialty)
        .bind(data.license_number)
        .bind(data.hospital_name)
        .bind(data.phone_professional)
        .bind(data.timezone)
        .fetch_one(pool)
        .await
    }
}
