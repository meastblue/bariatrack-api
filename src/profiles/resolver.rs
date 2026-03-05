use async_graphql::Context;
use sqlx::PgPool;

use super::entity::{Profile, SyncProfileInput, UpdateProfileInput};
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    /// Get current authenticated user profile
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<Profile> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_user_id(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile mutation first.")?;

        Ok(profile)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    /// Sync profile from Supabase Auth — must be called after first login
    async fn sync_profile(
        &self,
        ctx: &Context<'_>,
        data: SyncProfileInput,
    ) -> async_graphql::Result<Profile> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;
        let email = claims.email.clone().ok_or("Email not found in token")?;

        let profile = SyncProfileInput::sync(pool, auth_user_id, &email, data).await?;
        Ok(profile)
    }

    /// Update current user profile settings
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        data: UpdateProfileInput,
    ) -> async_graphql::Result<Profile> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = UpdateProfileInput::update(pool, auth_user_id, data).await?;
        Ok(profile)
    }

    /// Soft delete current user profile (RGPD)
    async fn delete_profile(&self, ctx: &Context<'_>) -> async_graphql::Result<Profile> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::delete(pool, auth_user_id).await?;
        Ok(profile)
    }
}
