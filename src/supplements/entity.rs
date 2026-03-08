use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "supplement_frequency", rename_all = "snake_case")]
pub enum SupplementFrequency {
    Daily,
    TwiceDaily,
    Weekly,
    AsNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Supplement {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: SupplementFrequency,
    pub prescribed_by: Option<Uuid>,
    pub is_active: bool,
    pub started_at: Option<NaiveDate>,
    pub ended_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct SupplementLog {
    pub id: Uuid,
    pub supplement_id: Uuid,
    pub patient_id: Uuid,
    pub taken_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, InputObject)]
pub struct CreateSupplementInput {
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: SupplementFrequency,
    pub started_at: Option<NaiveDate>,
    pub ended_at: Option<NaiveDate>,
}

#[derive(Debug, InputObject)]
pub struct UpdateSupplementInput {
    pub name: Option<String>,
    pub dosage: Option<String>,
    pub frequency: Option<SupplementFrequency>,
    pub is_active: Option<bool>,
    pub started_at: Option<NaiveDate>,
    pub ended_at: Option<NaiveDate>,
}

impl Supplement {
    pub async fn list_for_patient(pool: &PgPool, patient_id: Uuid) -> Result<Vec<Supplement>, sqlx::Error> {
        sqlx::query_as::<_, Supplement>(
            "SELECT * FROM supplements WHERE patient_id = $1 ORDER BY created_at DESC"
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &PgPool, patient_id: Uuid, doctor_id: Option<Uuid>, input: CreateSupplementInput) -> Result<Supplement, sqlx::Error> {
        sqlx::query_as::<_, Supplement>(
            r#"INSERT INTO supplements (patient_id, name, dosage, frequency, prescribed_by, started_at, ended_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#
        )
        .bind(patient_id)
        .bind(&input.name)
        .bind(&input.dosage)
        .bind(&input.frequency)
        .bind(doctor_id)
        .bind(input.started_at)
        .bind(input.ended_at)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &PgPool, id: Uuid, input: UpdateSupplementInput) -> Result<Supplement, sqlx::Error> {
        sqlx::query_as::<_, Supplement>(
            r#"UPDATE supplements SET
               name = COALESCE($2, name),
               dosage = COALESCE($3, dosage),
               frequency = COALESCE($4, frequency),
               is_active = COALESCE($5, is_active),
               started_at = COALESCE($6, started_at),
               ended_at = COALESCE($7, ended_at),
               updated_at = now()
               WHERE id = $1 RETURNING *"#
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.dosage)
        .bind(input.frequency)
        .bind(input.is_active)
        .bind(input.started_at)
        .bind(input.ended_at)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM supplements WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl SupplementLog {
    pub async fn list_for_patient(pool: &PgPool, patient_id: Uuid) -> Result<Vec<SupplementLog>, sqlx::Error> {
        sqlx::query_as::<_, SupplementLog>(
            "SELECT * FROM supplement_logs WHERE patient_id = $1 ORDER BY taken_at DESC"
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    pub async fn log_taken(pool: &PgPool, supplement_id: Uuid, patient_id: Uuid) -> Result<SupplementLog, sqlx::Error> {
        sqlx::query_as::<_, SupplementLog>(
            "INSERT INTO supplement_logs (supplement_id, patient_id) VALUES ($1, $2) RETURNING *"
        )
        .bind(supplement_id)
        .bind(patient_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM supplement_logs WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
