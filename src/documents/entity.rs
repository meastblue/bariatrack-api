use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "document_type", rename_all = "snake_case")]
pub enum DocumentType {
    LabResult,
    ScanMriCt,
    Prescription,
    SurgeryReport,
    ConsentForm,
    PhotoProgress,
    NutritionPlan,
    Other,
}

// ── Entity ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Document {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub uploaded_by: Uuid,
    pub appointment_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub file_url: String,
    pub file_type: DocumentType,
    pub mime_type: Option<String>,
    pub file_size_bytes: Option<i32>,
    pub uploaded_at: DateTime<Utc>,
}

// ── Input ──────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateDocumentInput {
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub file_url: String,
    pub file_type: DocumentType,
    pub mime_type: Option<String>,
    pub file_size_bytes: Option<i32>,
}

impl Document {
    /// Liste les documents d'un patient triés par date d'upload descendante
    pub async fn list_for_patient(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Vec<Document>, sqlx::Error> {
        sqlx::query_as::<_, Document>(
            "SELECT id, patient_id, uploaded_by, appointment_id, title, description, \
             file_url, file_type, mime_type, file_size_bytes, uploaded_at \
             FROM documents WHERE patient_id = $1 ORDER BY uploaded_at DESC",
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    /// Crée un document
    pub async fn create(
        pool: &PgPool,
        uploaded_by: Uuid,
        input: CreateDocumentInput,
    ) -> Result<Document, sqlx::Error> {
        sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (patient_id, uploaded_by, appointment_id, title, description, file_url, file_type, mime_type, file_size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, patient_id, uploaded_by, appointment_id, title, description,
                      file_url, file_type, mime_type, file_size_bytes, uploaded_at
            "#,
        )
        .bind(input.patient_id)
        .bind(uploaded_by)
        .bind(input.appointment_id)
        .bind(input.title)
        .bind(input.description)
        .bind(input.file_url)
        .bind(input.file_type)
        .bind(input.mime_type)
        .bind(input.file_size_bytes)
        .fetch_one(pool)
        .await
    }

    /// Supprime un document par id
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
