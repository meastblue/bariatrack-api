use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{CreateDoctorInput, Doctor, UpdateDoctorInput};
use crate::profiles::entity::Profile;
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct DoctorQuery;

#[async_graphql::Object]
impl DoctorQuery {
    /// Liste tous les médecins (admin)
    async fn doctors(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Doctor>> {
        let pool = ctx.data::<PgPool>()?;
        let doctors = Doctor::list(pool).await?;
        Ok(doctors)
    }

    /// Liste les médecins vérifiés (visible par tous les patients)
    async fn verified_doctors(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Doctor>> {
        let pool = ctx.data::<PgPool>()?;
        let doctors = Doctor::list_verified(pool).await?;
        Ok(doctors)
    }

    /// Récupère un médecin par id
    async fn doctor(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Doctor> {
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::get(pool, id).await?;
        Ok(doctor)
    }

    /// Récupère le profil médecin de l'utilisateur connecté
    async fn my_doctor(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Doctor>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let doctor = Doctor::find_by_profile_id(pool, profile.id).await?;
        Ok(doctor)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct DoctorMutation;

#[async_graphql::Object]
impl DoctorMutation {
    /// Crée le profil médecin de l'utilisateur connecté
    async fn create_doctor(
        &self,
        ctx: &Context<'_>,
        data: CreateDoctorInput,
    ) -> async_graphql::Result<Doctor> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let doctor = CreateDoctorInput::create(pool, profile.id, data).await?;
        Ok(doctor)
    }

    /// Met à jour un profil médecin
    async fn update_doctor(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        data: UpdateDoctorInput,
    ) -> async_graphql::Result<Doctor> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = UpdateDoctorInput::update(pool, id, data).await?;
        Ok(doctor)
    }

    /// Vérifie un médecin — admin only
    async fn verify_doctor(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Doctor> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::verify(pool, id).await?;
        Ok(doctor)
    }

    /// Soft delete — marque deleted_at
    async fn delete_doctor(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Doctor> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let doctor = Doctor::soft_delete(pool, id).await?;
        Ok(doctor)
    }

    /// Hard delete — suppression définitive (RGPD)
    async fn destroy_doctor(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Doctor::get_any(pool, id).await?;
        Doctor::destroy(pool, id).await?;
        Ok(true)
    }
}
