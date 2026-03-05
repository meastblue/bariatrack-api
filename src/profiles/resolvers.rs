use async_graphql::{Context, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::Profile;
use super::types::{ProfileType, SyncProfileInput, UpdateProfileInput};
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

fn get_auth_user_id(claims: &Claims) -> async_graphql::Result<Uuid> {
    claims
        .supabase_uid()
        .map_err(|_| "Invalid auth user ID".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct ProfileQuery;

#[Object]
impl ProfileQuery {
    /// Get current authenticated user profile
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<ProfileType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = get_auth_user_id(&claims)?;

        let profile = Profile::find_by_auth_user_id(pool, auth_user_id)
            .await?
            .ok_or("Profile not found. Call syncProfile mutation first.")?;

        Ok(profile.into())
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct ProfileMutation;

#[Object]
impl ProfileMutation {
    /// Sync profile from Supabase Auth into local database
    /// Must be called after first login to create the profile
    async fn sync_profile(
        &self,
        ctx: &Context<'_>,
        input: SyncProfileInput,
    ) -> async_graphql::Result<ProfileType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = get_auth_user_id(&claims)?;
        let email = claims.email.clone().ok_or("Email not found in token")?;

        let profile = Profile::upsert(pool, auth_user_id, &email, input.role.into()).await?;
        Ok(profile.into())
    }

    /// Update current user profile settings
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateProfileInput,
    ) -> async_graphql::Result<ProfileType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = get_auth_user_id(&claims)?;

        if let Some(locale) = input.locale {
            let profile = Profile::update_locale(pool, auth_user_id, locale)
                .await?
                .ok_or("Profile not found")?;
            return Ok(profile.into());
        }

        let profile = Profile::find_by_auth_user_id(pool, auth_user_id)
            .await?
            .ok_or("Profile not found")?;

        Ok(profile.into())
    }

    /// Soft delete current user profile (RGPD)
    async fn delete_profile(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_user_id = get_auth_user_id(&claims)?;

        let deleted = Profile::soft_delete(pool, auth_user_id).await?;
        Ok(deleted)
    }
}
