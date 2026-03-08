use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use uuid::Uuid;

// ── Enums ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    AppointmentReminder,
    AppointmentConfirmed,
    AppointmentCancelled,
    WeightGoalReached,
    HydrationReminder,
    SupplementReminder,
    NewDocument,
    DoctorMessage,
    PhaseChange,
    SystemAlert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Enum, Copy)]
#[sqlx(type_name = "reminder_period", rename_all = "snake_case")]
pub enum ReminderPeriod {
    Morning,
    Afternoon,
    Evening,
}

// ── Entities ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct Notification {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: Option<String>,
    pub is_read: bool,
    pub related_entity_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SimpleObject)]
pub struct NotificationPreference {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub notif_type: NotificationType,
    pub enabled: bool,
    pub periods: Vec<ReminderPeriod>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Input ──────────────────────────────────

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct UpdatePreferenceInput {
    pub enabled: bool,
    pub periods: Vec<ReminderPeriod>,
}

impl Notification {
    /// Liste les notifications d'un profil (filtrables par non-lues uniquement)
    pub async fn list_for_profile(
        pool: &PgPool,
        profile_id: Uuid,
        unread_only: bool,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        if unread_only {
            sqlx::query_as::<_, Notification>(
                "SELECT id, profile_id, notification_type, title, body, is_read, \
                 related_entity_id, related_entity_type, scheduled_at, sent_at, idempotency_key, created_at \
                 FROM notifications \
                 WHERE profile_id = $1 AND is_read = false \
                 ORDER BY created_at DESC",
            )
            .bind(profile_id)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, Notification>(
                "SELECT id, profile_id, notification_type, title, body, is_read, \
                 related_entity_id, related_entity_type, scheduled_at, sent_at, idempotency_key, created_at \
                 FROM notifications \
                 WHERE profile_id = $1 \
                 ORDER BY created_at DESC",
            )
            .bind(profile_id)
            .fetch_all(pool)
            .await
        }
    }

    /// Marque une notification comme lue
    pub async fn mark_read(pool: &PgPool, id: Uuid) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications SET is_read = true
            WHERE id = $1
            RETURNING id, profile_id, notification_type, title, body, is_read,
                      related_entity_id, related_entity_type, scheduled_at, sent_at, idempotency_key, created_at
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Marque toutes les notifications non-lues d'un profil comme lues
    pub async fn mark_all_read(pool: &PgPool, profile_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE profile_id = $1 AND is_read = false",
        )
        .bind(profile_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Crée une notification
    #[allow(dead_code)]
    pub async fn create(
        pool: &PgPool,
        profile_id: Uuid,
        notif_type: NotificationType,
        title: String,
        body: Option<String>,
        related_entity_id: Option<Uuid>,
        related_entity_type: Option<String>,
    ) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (profile_id, notification_type, title, body, related_entity_id, related_entity_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, profile_id, notification_type, title, body, is_read,
                      related_entity_id, related_entity_type, scheduled_at, sent_at, idempotency_key, created_at
            "#,
        )
        .bind(profile_id)
        .bind(notif_type)
        .bind(title)
        .bind(body)
        .bind(related_entity_id)
        .bind(related_entity_type)
        .fetch_one(pool)
        .await
    }
}

impl NotificationPreference {
    /// Liste toutes les préférences d'un patient
    pub async fn list_for_patient(
        pool: &PgPool,
        patient_id: Uuid,
    ) -> Result<Vec<NotificationPreference>, sqlx::Error> {
        sqlx::query_as::<_, NotificationPreference>(
            "SELECT id, patient_id, notif_type, enabled, periods, created_at, updated_at \
             FROM notification_preferences WHERE patient_id = $1 ORDER BY notif_type",
        )
        .bind(patient_id)
        .fetch_all(pool)
        .await
    }

    /// Upsert une préférence de notification
    pub async fn upsert(
        pool: &PgPool,
        patient_id: Uuid,
        notif_type: NotificationType,
        input: UpdatePreferenceInput,
    ) -> Result<NotificationPreference, sqlx::Error> {
        sqlx::query_as::<_, NotificationPreference>(
            r#"
            INSERT INTO notification_preferences (patient_id, notif_type, enabled, periods)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (patient_id, notif_type) DO UPDATE
                SET enabled = EXCLUDED.enabled,
                    periods = EXCLUDED.periods,
                    updated_at = NOW()
            RETURNING id, patient_id, notif_type, enabled, periods, created_at, updated_at
            "#,
        )
        .bind(patient_id)
        .bind(notif_type)
        .bind(input.enabled)
        .bind(input.periods)
        .fetch_one(pool)
        .await
    }
}
