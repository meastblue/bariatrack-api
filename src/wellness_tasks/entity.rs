use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "task_category", rename_all = "snake_case")]
pub enum TaskCategory {
    MentalHealth,
    Physical,
    Nutrition,
    Education,
    Breathing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Completed,
    Skipped,
}

// ── Entities ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct WellnessTask {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: TaskCategory,
    pub duration_min: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct PatientTask {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub task_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub status: TaskStatus,
    pub assigned_at: NaiveDate,
    pub completed_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl WellnessTask {
    /// Liste toutes les tâches actives du catalogue
    pub async fn list(pool: &PgPool) -> Result<Vec<WellnessTask>, sqlx::Error> {
        sqlx::query_as::<_, WellnessTask>(
            "SELECT id, title, description, category, duration_min, is_active, created_at \
             FROM wellness_tasks WHERE is_active = true ORDER BY created_at ASC",
        )
        .fetch_all(pool)
        .await
    }
}

impl PatientTask {
    /// Liste les tâches d'un patient, filtrées par date si fournie
    pub async fn list_for_patient(
        pool: &PgPool,
        patient_id: Uuid,
        date: Option<NaiveDate>,
    ) -> Result<Vec<PatientTask>, sqlx::Error> {
        if let Some(d) = date {
            sqlx::query_as::<_, PatientTask>(
                "SELECT id, patient_id, task_id, assigned_by, status, assigned_at, completed_at, note, created_at \
                 FROM patient_tasks \
                 WHERE patient_id = $1 AND assigned_at = $2 \
                 ORDER BY created_at ASC",
            )
            .bind(patient_id)
            .bind(d)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, PatientTask>(
                "SELECT id, patient_id, task_id, assigned_by, status, assigned_at, completed_at, note, created_at \
                 FROM patient_tasks \
                 WHERE patient_id = $1 \
                 ORDER BY assigned_at DESC, created_at ASC",
            )
            .bind(patient_id)
            .fetch_all(pool)
            .await
        }
    }

    /// Assigne une tâche à un patient
    pub async fn assign(
        pool: &PgPool,
        patient_id: Uuid,
        task_id: Uuid,
        doctor_id: Option<Uuid>,
        assigned_at: NaiveDate,
    ) -> Result<PatientTask, sqlx::Error> {
        sqlx::query_as::<_, PatientTask>(
            r#"
            INSERT INTO patient_tasks (patient_id, task_id, assigned_by, assigned_at, status)
            VALUES ($1, $2, $3, $4, 'pending')
            ON CONFLICT (patient_id, task_id, assigned_at) DO UPDATE
                SET assigned_by = EXCLUDED.assigned_by
            RETURNING id, patient_id, task_id, assigned_by, status, assigned_at, completed_at, note, created_at
            "#,
        )
        .bind(patient_id)
        .bind(task_id)
        .bind(doctor_id)
        .bind(assigned_at)
        .fetch_one(pool)
        .await
    }

    /// Marque une tâche comme complétée
    pub async fn complete(
        pool: &PgPool,
        id: Uuid,
        note: Option<String>,
    ) -> Result<PatientTask, sqlx::Error> {
        sqlx::query_as::<_, PatientTask>(
            r#"
            UPDATE patient_tasks
            SET status = 'completed', completed_at = NOW(), note = COALESCE($2, note)
            WHERE id = $1
            RETURNING id, patient_id, task_id, assigned_by, status, assigned_at, completed_at, note, created_at
            "#,
        )
        .bind(id)
        .bind(note)
        .fetch_one(pool)
        .await
    }

    /// Marque une tâche comme skippée
    pub async fn skip(pool: &PgPool, id: Uuid) -> Result<PatientTask, sqlx::Error> {
        sqlx::query_as::<_, PatientTask>(
            r#"
            UPDATE patient_tasks
            SET status = 'skipped'
            WHERE id = $1
            RETURNING id, patient_id, task_id, assigned_by, status, assigned_at, completed_at, note, created_at
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }
}
