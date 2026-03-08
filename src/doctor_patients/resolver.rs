use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::DoctorPatient;
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
pub struct DoctorPatientQuery;

#[async_graphql::Object]
impl DoctorPatientQuery {
    /// Selon le rôle : doctor → ses patients, patient → ses médecins
    async fn my_doctor_patients(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<DoctorPatient>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        match profile.role {
            UserRole::Doctor => {
                let doctor = Doctor::find_by_profile_id(pool, profile.id)
                    .await?
                    .ok_or("Doctor profile not found")?;
                let relations = DoctorPatient::list_patients_for_doctor(pool, doctor.id).await?;
                Ok(relations)
            }
            UserRole::Patient => {
                let patient = Patient::find_by_profile_id(pool, profile.id)
                    .await?
                    .ok_or("Patient profile not found")?;
                let relations = DoctorPatient::list_doctors_for_patient(pool, patient.id).await?;
                Ok(relations)
            }
            UserRole::Admin => {
                // Admin voit tout (toutes les relations actives)
                let relations = sqlx::query_as::<_, DoctorPatient>(
                    "SELECT * FROM doctor_patients WHERE status = 'active' ORDER BY assigned_at DESC",
                )
                .fetch_all(pool)
                .await?;
                Ok(relations)
            }
        }
    }

    /// Historique complet des relations pour un patient — admin only
    async fn doctor_patient_history(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
    ) -> async_graphql::Result<Vec<DoctorPatient>> {
        require_role(ctx, UserRole::Admin).await?;
        let pool = ctx.data::<PgPool>()?;
        let history = DoctorPatient::history_for_patient(pool, patient_id).await?;
        Ok(history)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct DoctorPatientMutation;

#[async_graphql::Object]
impl DoctorPatientMutation {
    /// Le médecin connecté s'assigne un patient
    async fn assign_patient(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
    ) -> async_graphql::Result<DoctorPatient> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found")?;
        let relation = DoctorPatient::assign(pool, doctor.id, patient_id).await?;
        Ok(relation)
    }

    /// Le médecin connecté retire un patient de son suivi
    async fn unassign_patient(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
    ) -> async_graphql::Result<DoctorPatient> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found")?;
        let relation = DoctorPatient::unassign(pool, doctor.id, patient_id).await?;
        Ok(relation)
    }

    /// Transférer un patient vers un autre médecin — le médecin connecté ou admin
    async fn transfer_patient(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
        new_doctor_id: Uuid,
        note: Option<String>,
    ) -> async_graphql::Result<DoctorPatient> {
        let profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Doctor profile not found")?;
        let relation =
            DoctorPatient::transfer(pool, patient_id, doctor.id, new_doctor_id, note).await?;
        Ok(relation)
    }
}
