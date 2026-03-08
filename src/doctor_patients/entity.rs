use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "relation_status", rename_all = "snake_case")]
pub enum RelationStatus {
    Active,
    Inactive,
    Transferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct DoctorPatient {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub status: RelationStatus,
    pub assigned_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transfer_note: Option<String>,
}

impl DoctorPatient {
    /// Assigner un médecin à un patient
    pub async fn assign(
        pool: &PgPool,
        doctor_id: Uuid,
        patient_id: Uuid,
    ) -> Result<DoctorPatient, sqlx::Error> {
        sqlx::query_as::<_, DoctorPatient>(
            r#"
            INSERT INTO doctor_patients (doctor_id, patient_id, status)
            VALUES ($1, $2, 'active')
            ON CONFLICT DO NOTHING
            RETURNING *
            "#,
        )
        .bind(doctor_id)
        .bind(patient_id)
        .fetch_one(pool)
        .await
    }

    /// Désassigner (status → inactive)
    pub async fn unassign(
        pool: &PgPool,
        doctor_id: Uuid,
        patient_id: Uuid,
    ) -> Result<DoctorPatient, sqlx::Error> {
        sqlx::query_as::<_, DoctorPatient>(
            r#"
            UPDATE doctor_patients
            SET status = 'inactive', ended_at = NOW()
            WHERE doctor_id = $1 AND patient_id = $2 AND status = 'active'
            RETURNING *
            "#,
        )
        .bind(doctor_id)
        .bind(patient_id)
        .fetch_one(pool)
        .await
    }

    /// Transférer un patient vers un autre médecin
    pub async fn transfer(
        pool: &PgPool,
        patient_id: Uuid,
        old_doctor_id: Uuid,
        new_doctor_id: Uuid,
        note: Option<String>,
    ) -> Result<DoctorPatient, sqlx::Error> {
        // Clore l'ancienne relation
        sqlx::query(
            r#"UPDATE doctor_patients SET status = 'transferred', ended_at = NOW(), transfer_note = $3
               WHERE doctor_id = $1 AND patient_id = $2 AND status = 'active'"#,
        )
        .bind(old_doctor_id)
        .bind(patient_id)
        .bind(&note)
        .execute(pool)
        .await?;

        // Créer la nouvelle
        sqlx::query_as::<_, DoctorPatient>(
            r#"INSERT INTO doctor_patients (doctor_id, patient_id, status)
               VALUES ($1, $2, 'active') RETURNING *"#,
        )
        .bind(new_doctor_id)
        .bind(patient_id)
        .fetch_one(pool)
        .await
    }

    /// Historique complet pour un patient
    pub async fn history_for_patient(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Vec<DoctorPatient>, sqlx::Error> {
        sqlx::query_as::<_, DoctorPatient>(
            "SELECT * FROM doctor_patients WHERE patient_id = $1 ORDER BY assigned_at DESC",
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    /// Liste des médecins d'un patient (actifs)
    pub async fn list_doctors_for_patient(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Vec<DoctorPatient>, sqlx::Error> {
        sqlx::query_as::<_, DoctorPatient>(
            "SELECT * FROM doctor_patients WHERE patient_id = $1 AND status = 'active' ORDER BY assigned_at DESC",
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    /// Liste des patients d'un médecin (actifs)
    pub async fn list_patients_for_doctor(
        pool: &PgPool,
        doctor_id: Uuid,
    ) -> Result<Vec<DoctorPatient>, sqlx::Error> {
        sqlx::query_as::<_, DoctorPatient>(
            "SELECT * FROM doctor_patients WHERE doctor_id = $1 AND status = 'active' ORDER BY assigned_at DESC",
        )
        .bind(doctor_id)
        .fetch_all(pool)
        .await
    }
}
