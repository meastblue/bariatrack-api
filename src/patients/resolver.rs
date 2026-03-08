use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{CreatePatientInput, Patient, UpdatePatientInput};
use crate::doctors::entity::Doctor;
use crate::profiles::entity::{Profile, UserRole};
use crate::utils::auth::{require_role, Claims};

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct PatientQuery;

#[async_graphql::Object]
impl PatientQuery {
    /// Liste tous les patients (admin only)
    async fn patients(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Patient>> {
        require_role(ctx, UserRole::Admin).await?;
        let pool = ctx.data::<PgPool>()?;
        let patients = Patient::list(pool).await?;
        Ok(patients)
    }

    /// Récupère un patient par id (admin only)
    async fn patient(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Patient> {
        require_role(ctx, UserRole::Admin).await?;
        let pool = ctx.data::<PgPool>()?;
        let patient = Patient::get(pool, id).await?;
        Ok(patient)
    }

    /// Liste les patients du médecin connecté (via doctor_patients)
    async fn my_patients(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Patient>> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;

        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found")?;

        let patients = sqlx::query_as::<_, Patient>(
            r#"
            SELECT p.* FROM patients p
            INNER JOIN doctor_patients dp ON dp.patient_id = p.id
            WHERE dp.doctor_id = $1
              AND dp.status = 'active'
              AND p.deleted_at IS NULL
            ORDER BY dp.assigned_at DESC
            "#,
        )
        .bind(doctor.id)
        .fetch_all(pool)
        .await?;

        Ok(patients)
    }

    /// Récupère le profil patient de l'utilisateur connecté
    async fn my_patient(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Patient>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id).await?;
        Ok(patient)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct PatientMutation;

#[async_graphql::Object]
impl PatientMutation {
    /// Crée le profil patient de l'utilisateur connecté
    async fn create_patient(
        &self,
        ctx: &Context<'_>,
        data: CreatePatientInput,
    ) -> async_graphql::Result<Patient> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = CreatePatientInput::create(pool, profile.id, data).await?;
        Ok(patient)
    }

    /// Met à jour un profil patient
    async fn update_patient(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        data: UpdatePatientInput,
    ) -> async_graphql::Result<Patient> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let patient = UpdatePatientInput::update(pool, id, data).await?;
        Ok(patient)
    }

    /// Soft delete — marque deleted_at
    async fn delete_patient(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Patient> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let patient = Patient::soft_delete(pool, id).await?;
        Ok(patient)
    }

    /// Hard delete — suppression définitive (RGPD)
    async fn destroy_patient(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Patient::get_any(pool, id).await?;
        Patient::destroy(pool, id).await?;
        Ok(true)
    }
}
