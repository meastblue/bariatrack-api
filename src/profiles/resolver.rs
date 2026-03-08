use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

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
    /// List all profiles (admin only)
    async fn list_profiles(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] include_deleted: bool,
    ) -> async_graphql::Result<Vec<Profile>> {
        let pool = ctx.data::<PgPool>()?;
        let profiles = Profile::list(pool, include_deleted).await?;
        Ok(profiles)
    }

    /// Get profile by id
    async fn get_profile(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Profile> {
        let pool = ctx.data::<PgPool>()?;
        let profile = Profile::get(pool, id).await?;
        Ok(profile)
    }

    /// Get current authenticated user profile
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<Profile> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_user_id(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        Ok(profile)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    /// Sync profile depuis Supabase Auth — appeler au premier login
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

    /// Update profile settings
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        data: UpdateProfileInput,
    ) -> async_graphql::Result<Profile> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let profile = UpdateProfileInput::update(pool, id, data).await?;
        Ok(profile)
    }

    /// Soft delete profile (RGPD)
    async fn delete_profile(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Profile> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let profile = Profile::delete(pool, id).await?;
        Ok(profile)
    }

    /// Hard delete profile — suppression définitive (RGPD purge)
    async fn destroy_profile(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Profile::destroy(pool, id).await?;
        Ok(true)
    }
}
