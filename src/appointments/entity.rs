use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "appointment_type", rename_all = "snake_case")]
pub enum AppointmentType {
    Consultation,
    FollowUp,
    Emergency,
    Teleconsultation,
    Nutritionist,
    Psychologist,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "appointment_status", rename_all = "snake_case")]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    Completed,
    Cancelled,
    NoShow,
    Rescheduled,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Option<Uuid>,
    pub appointment_type: AppointmentType,
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub duration_min: i32,
    pub location: Option<String>,
    pub teleconsult_url: Option<String>,
    pub status: AppointmentStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Appointment {
    pub async fn list_by_patient(
        pool: &PgPool,
        patient_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Appointment>, sqlx::Error> {
        sqlx::query_as::<_, Appointment>(
            r#"
            SELECT id, patient_id, doctor_id, appointment_type, title,
                   scheduled_at, duration_min, location, teleconsult_url,
                   status, notes, created_at, updated_at
            FROM appointments
            WHERE patient_id = $1
            ORDER BY scheduled_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(patient_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Appointment, sqlx::Error> {
        sqlx::query_as::<_, Appointment>(
            r#"
            SELECT id, patient_id, doctor_id, appointment_type, title,
                   scheduled_at, duration_min, location, teleconsult_url,
                   status, notes, created_at, updated_at
            FROM appointments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn cancel(pool: &PgPool, id: Uuid) -> Result<Appointment, sqlx::Error> {
        sqlx::query_as::<_, Appointment>(
            r#"
            UPDATE appointments SET
                status     = 'cancelled',
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, patient_id, doctor_id, appointment_type, title,
                      scheduled_at, duration_min, location, teleconsult_url,
                      status, notes, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM appointments WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

// ── Create ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateAppointmentInput {
    pub doctor_id: Option<Uuid>,
    pub appointment_type: AppointmentType,
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub duration_min: Option<i32>,
    pub location: Option<String>,
    pub teleconsult_url: Option<String>,
    pub notes: Option<String>,
}

impl CreateAppointmentInput {
    pub async fn create(
        pool: &PgPool,
        patient_id: Uuid,
        data: CreateAppointmentInput,
    ) -> Result<Appointment, sqlx::Error> {
        sqlx::query_as::<_, Appointment>(
            r#"
            INSERT INTO appointments (
                patient_id, doctor_id, appointment_type, title,
                scheduled_at, duration_min, location, teleconsult_url, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, patient_id, doctor_id, appointment_type, title,
                      scheduled_at, duration_min, location, teleconsult_url,
                      status, notes, created_at, updated_at
            "#,
        )
        .bind(patient_id)
        .bind(data.doctor_id)
        .bind(data.appointment_type)
        .bind(data.title)
        .bind(data.scheduled_at)
        .bind(data.duration_min.unwrap_or(30))
        .bind(data.location)
        .bind(data.teleconsult_url)
        .bind(data.notes)
        .fetch_one(pool)
        .await
    }
}

// ── Update ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdateAppointmentInput {
    pub doctor_id: Option<Uuid>,
    pub appointment_type: Option<AppointmentType>,
    pub title: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_min: Option<i32>,
    pub location: Option<String>,
    pub teleconsult_url: Option<String>,
    pub status: Option<AppointmentStatus>,
    pub notes: Option<String>,
}

impl UpdateAppointmentInput {
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        data: UpdateAppointmentInput,
    ) -> Result<Appointment, sqlx::Error> {
        sqlx::query_as::<_, Appointment>(
            r#"
            UPDATE appointments SET
                doctor_id        = COALESCE($2, doctor_id),
                appointment_type = COALESCE($3, appointment_type),
                title            = COALESCE($4, title),
                scheduled_at     = COALESCE($5, scheduled_at),
                duration_min     = COALESCE($6, duration_min),
                location         = COALESCE($7, location),
                teleconsult_url  = COALESCE($8, teleconsult_url),
                status           = COALESCE($9, status),
                notes            = COALESCE($10, notes),
                updated_at       = NOW()
            WHERE id = $1
            RETURNING id, patient_id, doctor_id, appointment_type, title,
                      scheduled_at, duration_min, location, teleconsult_url,
                      status, notes, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(data.doctor_id)
        .bind(data.appointment_type)
        .bind(data.title)
        .bind(data.scheduled_at)
        .bind(data.duration_min)
        .bind(data.location)
        .bind(data.teleconsult_url)
        .bind(data.status)
        .bind(data.notes)
        .fetch_one(pool)
        .await
    }
}
