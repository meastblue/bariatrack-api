use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "diet_phase_status", rename_all = "snake_case")]
pub enum DietPhaseStatus {
    Active,
    Completed,
    Skipped,
}

// ── Entities ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct DietPhase {
    pub id: Uuid,
    pub name: String,
    pub phase_order: i32,
    pub description: Option<String>,
    pub duration_days: Option<i32>,
    pub allowed_foods: Option<String>,
    pub forbidden_foods: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct PatientDietPhase {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub phase_id: Uuid,
    pub prescribed_by: Option<Uuid>,
    pub started_at: NaiveDate,
    pub ended_at: Option<NaiveDate>,
    pub status: DietPhaseStatus,
    pub created_at: DateTime<Utc>,
}

impl DietPhase {
    /// Liste toutes les phases du catalogue, triées par ordre
    pub async fn list(pool: &PgPool) -> Result<Vec<DietPhase>, sqlx::Error> {
        sqlx::query_as::<_, DietPhase>(
            "SELECT id, name, phase_order, description, duration_days, allowed_foods, forbidden_foods \
             FROM diet_phases ORDER BY phase_order ASC",
        )
        .fetch_all(pool)
        .await
    }

    /// Récupère une phase du catalogue par id
    #[allow(dead_code)]
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<DietPhase, sqlx::Error> {
        sqlx::query_as::<_, DietPhase>(
            "SELECT id, name, phase_order, description, duration_days, allowed_foods, forbidden_foods \
             FROM diet_phases WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }
}

impl PatientDietPhase {
    /// Phase active du patient (status = 'active')
    pub async fn current(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Option<PatientDietPhase>, sqlx::Error> {
        sqlx::query_as::<_, PatientDietPhase>(
            "SELECT id, patient_id, phase_id, prescribed_by, started_at, ended_at, status, created_at \
             FROM patient_diet_phases \
             WHERE patient_id = $1 AND status = 'active' \
             ORDER BY started_at DESC \
             LIMIT 1",
        )
        .bind(patient_id)
        .fetch_optional(pool)
        .await
    }

    /// Historique de toutes les phases du patient
    pub async fn history(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Vec<PatientDietPhase>, sqlx::Error> {
        sqlx::query_as::<_, PatientDietPhase>(
            "SELECT id, patient_id, phase_id, prescribed_by, started_at, ended_at, status, created_at \
             FROM patient_diet_phases \
             WHERE patient_id = $1 \
             ORDER BY started_at DESC",
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    /// Assigne une phase à un patient
    pub async fn assign(
        pool: &PgPool,
        patient_id: Uuid,
        phase_id: Uuid,
        doctor_id: Option<Uuid>,
        started_at: NaiveDate,
    ) -> Result<PatientDietPhase, sqlx::Error> {
        sqlx::query_as::<_, PatientDietPhase>(
            r#"
            INSERT INTO patient_diet_phases (patient_id, phase_id, prescribed_by, started_at, status)
            VALUES ($1, $2, $3, $4, 'active')
            RETURNING id, patient_id, phase_id, prescribed_by, started_at, ended_at, status, created_at
            "#,
        )
        .bind(patient_id)
        .bind(phase_id)
        .bind(doctor_id)
        .bind(started_at)
        .fetch_one(pool)
        .await
    }

    /// Marque une phase comme complétée
    pub async fn complete(
        pool: &PgPool,
        id: Uuid,
        ended_at: NaiveDate,
    ) -> Result<PatientDietPhase, sqlx::Error> {
        sqlx::query_as::<_, PatientDietPhase>(
            r#"
            UPDATE patient_diet_phases
            SET status = 'completed', ended_at = $2
            WHERE id = $1
            RETURNING id, patient_id, phase_id, prescribed_by, started_at, ended_at, status, created_at
            "#,
        )
        .bind(id)
        .bind(ended_at)
        .fetch_one(pool)
        .await
    }

    /// Marque une phase comme skippée
    pub async fn skip(pool: &PgPool, id: Uuid) -> Result<PatientDietPhase, sqlx::Error> {
        sqlx::query_as::<_, PatientDietPhase>(
            r#"
            UPDATE patient_diet_phases
            SET status = 'skipped'
            WHERE id = $1
            RETURNING id, patient_id, phase_id, prescribed_by, started_at, ended_at, status, created_at
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }
}
