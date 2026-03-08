use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{Appointment, CreateAppointmentInput, UpdateAppointmentInput};
use crate::patients::entity::Patient;
use crate::profiles::entity::Profile;
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct AppointmentQuery;

#[async_graphql::Object]
impl AppointmentQuery {
    /// Liste paginée des rendez-vous du patient connecté
    async fn my_appointments(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<Appointment>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let appointments = Appointment::list_by_patient(
            pool,
            patient.id,
            limit.unwrap_or(20),
            offset.unwrap_or(0),
        )
        .await?;
        Ok(appointments)
    }

    /// Liste paginée des rendez-vous d'un patient (pour médecin)
    async fn patient_appointments(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<Appointment>> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;

        let appointments = Appointment::list_by_patient(
            pool,
            patient_id,
            limit.unwrap_or(20),
            offset.unwrap_or(0),
        )
        .await?;
        Ok(appointments)
    }

    /// Récupère un rendez-vous par id
    async fn appointment(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Appointment> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let appointment = Appointment::get(pool, id).await?;
        Ok(appointment)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct AppointmentMutation;

#[async_graphql::Object]
impl AppointmentMutation {
    /// Crée un rendez-vous pour le patient connecté
    async fn create_appointment(
        &self,
        ctx: &Context<'_>,
        data: CreateAppointmentInput,
    ) -> async_graphql::Result<Appointment> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let appointment = CreateAppointmentInput::create(pool, patient.id, data).await?;
        Ok(appointment)
    }

    /// Met à jour un rendez-vous
    async fn update_appointment(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        data: UpdateAppointmentInput,
    ) -> async_graphql::Result<Appointment> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let appointment = UpdateAppointmentInput::update(pool, id, data).await?;
        Ok(appointment)
    }

    /// Annule un rendez-vous (status → cancelled)
    async fn cancel_appointment(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Appointment> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let appointment = Appointment::cancel(pool, id).await?;
        Ok(appointment)
    }

    /// Supprime un rendez-vous
    async fn delete_appointment(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Appointment::delete(pool, id).await?;
        Ok(true)
    }
}
