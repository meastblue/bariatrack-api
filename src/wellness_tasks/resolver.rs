use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{PatientTask, WellnessTask};
use crate::doctors::entity::Doctor;
use crate::patients::entity::Patient;
use crate::profiles::entity::{Profile, UserRole};
use crate::utils::auth::{require_role, Claims};

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct WellnessTaskQuery;

#[async_graphql::Object]
impl WellnessTaskQuery {
    /// Liste le catalogue des tâches bien-être actives
    async fn wellness_tasks(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<WellnessTask>> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let tasks = WellnessTask::list(pool).await?;
        Ok(tasks)
    }

    /// Tâches du patient connecté pour une date donnée (format YYYY-MM-DD)
    async fn my_tasks(
        &self,
        ctx: &Context<'_>,
        date: Option<String>,
    ) -> async_graphql::Result<Vec<PatientTask>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let parsed_date = if let Some(s) = date {
            Some(
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map_err(|_| "Invalid date format, expected YYYY-MM-DD")?,
            )
        } else {
            None
        };

        let tasks = PatientTask::list_for_patient(pool, patient.id, parsed_date).await?;
        Ok(tasks)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct WellnessTaskMutation;

#[async_graphql::Object]
impl WellnessTaskMutation {
    /// Assigne une tâche bien-être à un patient (réservé aux médecins)
    async fn assign_task(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
        task_id: Uuid,
        assigned_at: Option<String>,
    ) -> async_graphql::Result<PatientTask> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;

        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found.")?;

        let date = if let Some(s) = assigned_at {
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|_| "Invalid date format, expected YYYY-MM-DD")?
        } else {
            chrono::Local::now().naive_local().date()
        };

        let task = PatientTask::assign(pool, patient_id, task_id, Some(doctor.id), date).await?;
        Ok(task)
    }

    /// Marque une tâche comme complétée par le patient connecté
    async fn complete_task(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        note: Option<String>,
    ) -> async_graphql::Result<PatientTask> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let task = PatientTask::complete(pool, id, note).await?;
        Ok(task)
    }

    /// Marque une tâche comme skippée par le patient connecté
    async fn skip_task(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<PatientTask> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let task = PatientTask::skip(pool, id).await?;
        Ok(task)
    }
}
