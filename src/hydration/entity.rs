use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::weights::entity::EntrySource;

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Hydration {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub total_ml: i32,
    pub goal_ml: i32,
    pub logged_at: NaiveDate,
    pub source: EntrySource,
    pub created_at: DateTime<Utc>,
}

impl Hydration {
    pub async fn list_by_patient(
        pool: &PgPool,
        patient_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Hydration>, sqlx::Error> {
        sqlx::query_as::<_, Hydration>(
            r#"
            SELECT id, patient_id, total_ml, goal_ml, logged_at, source, created_at
            FROM hydration_daily
            WHERE patient_id = $1
            ORDER BY logged_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(patient_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM hydration_daily WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Log (upsert) ───────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct LogHydrationInput {
    pub total_ml: i32,
    pub goal_ml: Option<i32>,
    pub logged_at: NaiveDate,
    pub source: Option<EntrySource>,
}

impl LogHydrationInput {
    pub async fn upsert(
        pool: &PgPool,
        patient_id: Uuid,
        data: LogHydrationInput,
    ) -> Result<Hydration, sqlx::Error> {
        sqlx::query_as::<_, Hydration>(
            r#"
            INSERT INTO hydration_daily (patient_id, total_ml, goal_ml, logged_at, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (patient_id, logged_at) DO UPDATE SET
                total_ml = EXCLUDED.total_ml,
                goal_ml  = EXCLUDED.goal_ml,
                source   = EXCLUDED.source
            RETURNING id, patient_id, total_ml, goal_ml, logged_at, source, created_at
            "#,
        )
        .bind(patient_id)
        .bind(data.total_ml)
        .bind(data.goal_ml.unwrap_or(1500))
        .bind(data.logged_at)
        .bind(data.source.unwrap_or(EntrySource::Manual))
        .fetch_one(pool)
        .await
    }
}
