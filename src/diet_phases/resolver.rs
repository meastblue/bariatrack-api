use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{DietPhase, PatientDietPhase};
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
pub struct DietPhaseQuery;

#[async_graphql::Object]
impl DietPhaseQuery {
    /// Liste le catalogue de toutes les phases diététiques, triées par ordre
    async fn diet_phases(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<DietPhase>> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let phases = DietPhase::list(pool).await?;
        Ok(phases)
    }

    /// Phase diététique active du patient connecté (Option)
    async fn my_diet_phase(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<PatientDietPhase>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let phase = PatientDietPhase::current(pool, patient.id).await?;
        Ok(phase)
    }

    /// Historique de toutes les phases diététiques du patient connecté
    async fn my_diet_phase_history(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<PatientDietPhase>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let history = PatientDietPhase::history(pool, patient.id).await?;
        Ok(history)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct DietPhaseMutation;

#[async_graphql::Object]
impl DietPhaseMutation {
    /// Assigne une phase diététique à un patient (réservé aux médecins)
    async fn assign_diet_phase(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
        phase_id: Uuid,
        started_at: Option<String>,
    ) -> async_graphql::Result<PatientDietPhase> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;

        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found.")?;

        let date = if let Some(s) = started_at {
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|_| "Invalid date format, expected YYYY-MM-DD")?
        } else {
            chrono::Local::now().naive_local().date()
        };

        let phase =
            PatientDietPhase::assign(pool, patient_id, phase_id, Some(doctor.id), date).await?;
        Ok(phase)
    }

    /// Marque une phase diététique comme complétée (réservé aux médecins)
    async fn complete_diet_phase(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        ended_at: Option<String>,
    ) -> async_graphql::Result<PatientDietPhase> {
        let _profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;

        let date = if let Some(s) = ended_at {
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|_| "Invalid date format, expected YYYY-MM-DD")?
        } else {
            chrono::Local::now().naive_local().date()
        };

        let phase = PatientDietPhase::complete(pool, id, date).await?;
        Ok(phase)
    }

    /// Marque une phase diététique comme skippée (réservé aux médecins)
    async fn skip_diet_phase(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<PatientDietPhase> {
        let _profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;

        let phase = PatientDietPhase::skip(pool, id).await?;
        Ok(phase)
    }
}
