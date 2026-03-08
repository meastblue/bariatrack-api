use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "mood_value", rename_all = "snake_case")]
pub enum MoodValue {
    Terrible,
    Bad,
    Neutral,
    Good,
    Awesome,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Mood {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub mood: MoodValue,
    pub note: Option<String>,
    pub logged_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl Mood {
    pub async fn list_by_patient(
        pool: &PgPool,
        patient_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Mood>, sqlx::Error> {
        sqlx::query_as::<_, Mood>(
            r#"
            SELECT id, patient_id, mood, note, logged_at, created_at
            FROM moods
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
        sqlx::query("DELETE FROM moods WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Log (upsert) ───────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct LogMoodInput {
    pub mood: MoodValue,
    pub logged_at: NaiveDate,
    pub note: Option<String>,
}

impl LogMoodInput {
    pub async fn upsert(
        pool: &PgPool,
        patient_id: Uuid,
        data: LogMoodInput,
    ) -> Result<Mood, sqlx::Error> {
        sqlx::query_as::<_, Mood>(
            r#"
            INSERT INTO moods (patient_id, mood, logged_at, note)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (patient_id, logged_at) DO UPDATE SET
                mood = EXCLUDED.mood,
                note = EXCLUDED.note
            RETURNING id, patient_id, mood, note, logged_at, created_at
            "#,
        )
        .bind(patient_id)
        .bind(data.mood)
        .bind(data.logged_at)
        .bind(data.note)
        .fetch_one(pool)
        .await
    }
}
