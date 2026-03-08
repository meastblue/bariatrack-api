use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{Notification, NotificationPreference, NotificationType, UpdatePreferenceInput};
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
pub struct NotificationQuery;

#[async_graphql::Object]
impl NotificationQuery {
    /// Liste les notifications du profil connecté. Filtrable par non-lues uniquement.
    async fn my_notifications(
        &self,
        ctx: &Context<'_>,
        unread_only: Option<bool>,
    ) -> async_graphql::Result<Vec<Notification>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let notifications =
            Notification::list_for_profile(pool, profile.id, unread_only.unwrap_or(false)).await?;
        Ok(notifications)
    }

    /// Liste les préférences de notification du patient connecté
    async fn my_notification_preferences(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<NotificationPreference>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let prefs = NotificationPreference::list_for_patient(pool, patient.id).await?;
        Ok(prefs)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct NotificationMutation;

#[async_graphql::Object]
impl NotificationMutation {
    /// Marque une notification comme lue
    async fn mark_notification_read(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Notification> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let notif = Notification::mark_read(pool, id).await?;
        Ok(notif)
    }

    /// Marque toutes les notifications non-lues du profil connecté comme lues
    async fn mark_all_notifications_read(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        Notification::mark_all_read(pool, profile.id).await?;
        Ok(true)
    }

    /// Met à jour ou crée une préférence de notification pour le patient connecté
    async fn update_notification_preference(
        &self,
        ctx: &Context<'_>,
        notif_type: NotificationType,
        data: UpdatePreferenceInput,
    ) -> async_graphql::Result<NotificationPreference> {
        let profile = require_role(ctx, UserRole::Patient).await?;
        let pool = ctx.data::<PgPool>()?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let pref = NotificationPreference::upsert(pool, patient.id, notif_type, data).await?;
        Ok(pref)
    }
}
