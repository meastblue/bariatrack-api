use async_graphql::{Context, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::User;
use super::types::UserType;
use crate::utils::auth::Claims;

fn get_claims(ctx: &Context<'_>) -> async_graphql::Result<Claims> {
    ctx.data::<Claims>()
        .map(|c| c.clone())
        .map_err(|_| "Unauthorized: missing or invalid token".into())
}

fn get_supabase_uid(claims: &Claims) -> async_graphql::Result<Uuid> {
    claims
        .supabase_uid()
        .map_err(|_| "Invalid supabase user ID".into())
}

// ── Query ──────────────────────────────────

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Get current authenticated user
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<UserType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let supabase_uid = get_supabase_uid(&claims)?;

        let user = User::find_by_supabase_uid(pool, supabase_uid)
            .await?
            .ok_or("User not found. Call syncUser mutation first.")?;

        Ok(user.into())
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    /// Sync user from Supabase Auth into local database
    async fn sync_user(&self, ctx: &Context<'_>) -> async_graphql::Result<UserType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let supabase_uid = get_supabase_uid(&claims)?;
        let email = claims.email.clone().ok_or("Email not found in token")?;

        let user = User::upsert(pool, supabase_uid, &email).await?;
        Ok(user.into())
    }

    /// Update current user's profile
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        username: Option<String>,
        avatar_url: Option<String>,
    ) -> async_graphql::Result<UserType> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let supabase_uid = get_supabase_uid(&claims)?;

        let user = User::update_profile(pool, supabase_uid, username, avatar_url)
            .await?
            .ok_or("User not found")?;

        Ok(user.into())
    }
}
