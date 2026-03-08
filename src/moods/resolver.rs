use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{LogMoodInput, Mood};
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
pub struct MoodQuery;

#[async_graphql::Object]
impl MoodQuery {
    /// Liste paginée des humeurs de l'utilisateur connecté
    async fn my_moods(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> async_graphql::Result<Vec<Mood>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let moods = Mood::list_by_patient(
            pool,
            patient.id,
            limit.unwrap_or(20),
            offset.unwrap_or(0),
        )
        .await?;
        Ok(moods)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct MoodMutation;

#[async_graphql::Object]
impl MoodMutation {
    /// Enregistre une humeur (upsert par date)
    async fn log_mood(
        &self,
        ctx: &Context<'_>,
        data: LogMoodInput,
    ) -> async_graphql::Result<Mood> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let mood = LogMoodInput::upsert(pool, patient.id, data).await?;
        Ok(mood)
    }

    /// Supprime une humeur
    async fn delete_mood(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Mood::delete(pool, id).await?;
        Ok(true)
    }
}
