use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{LogWeightInput, Weight};
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
pub struct WeightQuery;

#[async_graphql::Object]
impl WeightQuery {
    /// Liste paginée des pesées de l'utilisateur connecté
    async fn my_weights(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<Weight>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let weights = Weight::list_by_patient(
            pool,
            patient.id,
            limit.unwrap_or(20),
            offset.unwrap_or(0),
        )
        .await?;
        Ok(weights)
    }

    /// Récupère une pesée par id
    async fn weight(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Weight> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let weight = Weight::get(pool, id).await?;
        Ok(weight)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct WeightMutation;

#[async_graphql::Object]
impl WeightMutation {
    /// Enregistre une pesée (upsert par date)
    async fn log_weight(
        &self,
        ctx: &Context<'_>,
        data: LogWeightInput,
    ) -> async_graphql::Result<Weight> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let weight = LogWeightInput::upsert(pool, patient.id, data).await?;
        Ok(weight)
    }

    /// Supprime une pesée
    async fn delete_weight(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Weight::delete(pool, id).await?;
        Ok(true)
    }
}
