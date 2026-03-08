use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "surgery_type", rename_all = "snake_case")]
pub enum SurgeryType {
    SleeveGastrectomy,
    GastricBypassRygb,
    GastricBand,
    DuodenalSwitch,
    MiniBypassOagb,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "gender", rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    NonBinary,
    PreferNotToSay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "blood_type", rename_all = "snake_case")]
pub enum BloodType {
    #[sqlx(rename = "A_pos")]
    APos,
    #[sqlx(rename = "A_neg")]
    ANeg,
    #[sqlx(rename = "B_pos")]
    BPos,
    #[sqlx(rename = "B_neg")]
    BNeg,
    #[sqlx(rename = "AB_pos")]
    AbPos,
    #[sqlx(rename = "AB_neg")]
    AbNeg,
    #[sqlx(rename = "O_pos")]
    OPos,
    #[sqlx(rename = "O_neg")]
    ONeg,
    Unknown,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Patient {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub surgery_type: SurgeryType,
    pub surgery_date: NaiveDate,
    pub initial_weight_kg: f64,
    pub target_weight_kg: Option<f64>,
    pub height_cm: f64,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
    pub blood_type: Option<BloodType>,
    pub primary_doctor_id: Option<Uuid>,
    pub allergies: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Requête SELECT explicite avec cast NUMERIC → float8
const SELECT: &str = r#"
    SELECT
        id, profile_id, surgery_type, surgery_date,
        initial_weight_kg::float8 AS initial_weight_kg,
        target_weight_kg::float8  AS target_weight_kg,
        height_cm::float8         AS height_cm,
        date_of_birth, gender, blood_type,
        primary_doctor_id, allergies, notes,
        created_at, updated_at
    FROM patients
"#;

impl Patient {
    /// Liste tous les patients (admin)
    pub async fn list(pool: &PgPool) -> Result<Vec<Patient>, sqlx::Error> {
        let q = format!("{} ORDER BY created_at DESC", SELECT);
        sqlx::query_as::<_, Patient>(&q).fetch_all(pool).await
    }

    /// Récupère un patient par id
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Patient, sqlx::Error> {
        let q = format!("{} WHERE id = $1", SELECT);
        sqlx::query_as::<_, Patient>(&q)
            .bind(id)
            .fetch_one(pool)
            .await
    }

    /// Récupère le patient lié à un profil
    pub async fn find_by_profile_id(
        pool: &PgPool,
        profile_id: Uuid,
    ) -> Result<Option<Patient>, sqlx::Error> {
        let q = format!("{} WHERE profile_id = $1", SELECT);
        sqlx::query_as::<_, Patient>(&q)
            .bind(profile_id)
            .fetch_optional(pool)
            .await
    }

    /// Supprime un patient (hard delete — pas de soft delete sur cette table)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM patients WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Create ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreatePatientInput {
    pub surgery_type: SurgeryType,
    pub surgery_date: NaiveDate,
    pub initial_weight_kg: f64,
    pub target_weight_kg: Option<f64>,
    pub height_cm: f64,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
    pub blood_type: Option<BloodType>,
    pub primary_doctor_id: Option<Uuid>,
    pub allergies: Option<String>,
    pub notes: Option<String>,
}

impl CreatePatientInput {
    pub async fn create(
        pool: &PgPool,
        profile_id: Uuid,
        data: CreatePatientInput,
    ) -> Result<Patient, sqlx::Error> {
        sqlx::query_as::<_, Patient>(
            r#"
            INSERT INTO patients (
                profile_id, surgery_type, surgery_date,
                initial_weight_kg, target_weight_kg, height_cm,
                date_of_birth, gender, blood_type,
                primary_doctor_id, allergies, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, profile_id, surgery_type, surgery_date,
                initial_weight_kg::float8 AS initial_weight_kg,
                target_weight_kg::float8  AS target_weight_kg,
                height_cm::float8         AS height_cm,
                date_of_birth, gender, blood_type,
                primary_doctor_id, allergies, notes,
                created_at, updated_at
            "#,
        )
        .bind(profile_id)
        .bind(data.surgery_type)
        .bind(data.surgery_date)
        .bind(data.initial_weight_kg)
        .bind(data.target_weight_kg)
        .bind(data.height_cm)
        .bind(data.date_of_birth)
        .bind(data.gender)
        .bind(data.blood_type)
        .bind(data.primary_doctor_id)
        .bind(data.allergies)
        .bind(data.notes)
        .fetch_one(pool)
        .await
    }
}

// ── Update ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdatePatientInput {
    pub surgery_type: Option<SurgeryType>,
    pub surgery_date: Option<NaiveDate>,
    pub target_weight_kg: Option<f64>,
    pub height_cm: Option<f64>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
    pub blood_type: Option<BloodType>,
    pub primary_doctor_id: Option<Uuid>,
    pub allergies: Option<String>,
    pub notes: Option<String>,
}

impl UpdatePatientInput {
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: UpdatePatientInput,
    ) -> Result<Patient, sqlx::Error> {
        sqlx::query_as::<_, Patient>(
            r#"
            UPDATE patients SET
                surgery_type      = COALESCE($2, surgery_type),
                surgery_date      = COALESCE($3, surgery_date),
                target_weight_kg  = COALESCE($4, target_weight_kg),
                height_cm         = COALESCE($5, height_cm),
                date_of_birth     = COALESCE($6, date_of_birth),
                gender            = COALESCE($7, gender),
                blood_type        = COALESCE($8, blood_type),
                primary_doctor_id = COALESCE($9, primary_doctor_id),
                allergies         = COALESCE($10, allergies),
                notes             = COALESCE($11, notes),
                updated_at        = NOW()
            WHERE id = $1
            RETURNING
                id, profile_id, surgery_type, surgery_date,
                initial_weight_kg::float8 AS initial_weight_kg,
                target_weight_kg::float8  AS target_weight_kg,
                height_cm::float8         AS height_cm,
                date_of_birth, gender, blood_type,
                primary_doctor_id, allergies, notes,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(data.surgery_type)
        .bind(data.surgery_date)
        .bind(data.target_weight_kg)
        .bind(data.height_cm)
        .bind(data.date_of_birth)
        .bind(data.gender)
        .bind(data.blood_type)
        .bind(data.primary_doctor_id)
        .bind(data.allergies)
        .bind(data.notes)
        .fetch_one(pool)
        .await
    }
}
