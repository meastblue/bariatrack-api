use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "entry_source", rename_all = "snake_case")]
pub enum EntrySource {
    Manual,
    Healthkit,
    Doctor,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Weight {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub weight_kg: f64,
    pub measured_at: NaiveDate,
    pub source: EntrySource,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

const SELECT: &str = r#"
    SELECT
        id, patient_id,
        weight_kg::float8 AS weight_kg,
        measured_at, source, note, created_at
    FROM weights
"#;

impl Weight {
    pub async fn list_by_patient(
        pool: &PgPool,
        patient_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Weight>, sqlx::Error> {
        let q = format!(
            "{} WHERE patient_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
            SELECT
        );
        sqlx::query_as::<_, Weight>(&q)
            .bind(patient_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Weight, sqlx::Error> {
        let q = format!("{} WHERE id = $1", SELECT);
        sqlx::query_as::<_, Weight>(&q)
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM weights WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Log (upsert) ───────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct LogWeightInput {
    pub weight_kg: f64,
    pub measured_at: NaiveDate,
    pub source: Option<EntrySource>,
    pub note: Option<String>,
}

impl LogWeightInput {
    pub async fn upsert(
        pool: &PgPool,
        patient_id: Uuid,
        data: LogWeightInput,
    ) -> Result<Weight, sqlx::Error> {
        sqlx::query_as::<_, Weight>(
            r#"
            INSERT INTO weights (patient_id, weight_kg, measured_at, source, note)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (patient_id, measured_at) DO UPDATE SET
                weight_kg = EXCLUDED.weight_kg,
                source    = EXCLUDED.source,
                note      = EXCLUDED.note
            RETURNING
                id, patient_id,
                weight_kg::float8 AS weight_kg,
                measured_at, source, note, created_at
            "#,
        )
        .bind(patient_id)
        .bind(data.weight_kg)
        .bind(data.measured_at)
        .bind(data.source.unwrap_or(EntrySource::Manual))
        .bind(data.note)
        .fetch_one(pool)
        .await
    }
}
