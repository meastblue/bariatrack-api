use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{CreateSupplementInput, Supplement, SupplementLog, UpdateSupplementInput};
use crate::doctors::entity::Doctor;
use crate::patients::entity::Patient;
use crate::profiles::entity::{Profile, UserRole};
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized".into())
}

async fn get_patient_from_ctx(ctx: &Context<'_>) -> async_graphql::Result<Patient> {
    let claims = get_claims(ctx)?;
    let pool = ctx.data::<PgPool>()?;
    let auth_uid = claims.supabase_uid().map_err(|_| async_graphql::Error::new("Invalid token"))?;
    let profile = Profile::find_by_auth_uid(pool, auth_uid)
        .await?
        .ok_or("Profile not found")?;
    let patient = Patient::find_by_profile_id(pool, profile.id)
        .await?
        .ok_or("Patient profile not found")?;
    Ok(patient)
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct SupplementQuery;

#[async_graphql::Object]
impl SupplementQuery {
    /// Liste les suppléments du patient connecté
    async fn my_supplements(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Supplement>> {
        let pool = ctx.data::<PgPool>()?;
        let patient = get_patient_from_ctx(ctx).await?;
        let supplements = Supplement::list_for_patient(pool, patient.id).await?;
        Ok(supplements)
    }

    /// Liste les logs de prise du patient connecté
    async fn my_supplement_logs(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<SupplementLog>> {
        let pool = ctx.data::<PgPool>()?;
        let patient = get_patient_from_ctx(ctx).await?;
        let logs = SupplementLog::list_for_patient(pool, patient.id).await?;
        Ok(logs)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct SupplementMutation;

#[async_graphql::Object]
impl SupplementMutation {
    /// Crée un supplément — doctor (prescribed_by auto) ou patient
    async fn create_supplement(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
        data: CreateSupplementInput,
    ) -> async_graphql::Result<Supplement> {
        let claims = ctx
            .data::<Claims>()
            .map(|c| c.clone())
            .map_err(|_| "Unauthorized")?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid token")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found")?;

        let doctor_id = if profile.role == UserRole::Doctor {
            Doctor::find_by_profile_id(pool, profile.id)
                .await?
                .map(|d| d.id)
        } else {
            None
        };

        let supplement = Supplement::create(pool, patient_id, doctor_id, data).await?;
        Ok(supplement)
    }

    /// Met à jour un supplément
    async fn update_supplement(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        data: UpdateSupplementInput,
    ) -> async_graphql::Result<Supplement> {
        let _claims = ctx
            .data::<Claims>()
            .map_err(|_| "Unauthorized")?;
        let pool = ctx.data::<PgPool>()?;
        let supplement = Supplement::update(pool, id, data).await?;
        Ok(supplement)
    }

    /// Supprime un supplément
    async fn delete_supplement(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = ctx
            .data::<Claims>()
            .map_err(|_| "Unauthorized")?;
        let pool = ctx.data::<PgPool>()?;
        Supplement::delete(pool, id).await?;
        Ok(true)
    }

    /// Le patient connecté enregistre une prise de supplément
    async fn log_supplement_taken(
        &self,
        ctx: &Context<'_>,
        supplement_id: Uuid,
    ) -> async_graphql::Result<SupplementLog> {
        let pool = ctx.data::<PgPool>()?;
        let patient = get_patient_from_ctx(ctx).await?;
        let log = SupplementLog::log_taken(pool, supplement_id, patient.id).await?;
        Ok(log)
    }

    /// Supprime un log de prise
    async fn delete_supplement_log(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<bool> {
        let _claims = ctx
            .data::<Claims>()
            .map_err(|_| "Unauthorized")?;
        let pool = ctx.data::<PgPool>()?;
        SupplementLog::delete(pool, id).await?;
        Ok(true)
    }
}
