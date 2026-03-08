use async_graphql::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::entity::{CreateDocumentInput, Document};
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
pub struct DocumentQuery;

#[async_graphql::Object]
impl DocumentQuery {
    /// Liste les documents médicaux du patient connecté
    async fn my_documents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Document>> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let patient = Patient::find_by_profile_id(pool, profile.id)
            .await?
            .ok_or("Patient profile not found.")?;

        let docs = Document::list_for_patient(pool, patient.id).await?;
        Ok(docs)
    }

    /// Liste les documents d'un patient par son id (médecin ou admin uniquement)
    async fn patient_documents(
        &self,
        ctx: &Context<'_>,
        patient_id: Uuid,
    ) -> async_graphql::Result<Vec<Document>> {
        let _profile = require_role(ctx, UserRole::Doctor).await?;
        let pool = ctx.data::<PgPool>()?;
        let docs = Document::list_for_patient(pool, patient_id).await?;
        Ok(docs)
    }
}

// ── Mutation ───────────────────────────────

#[derive(Default)]
pub struct DocumentMutation;

#[async_graphql::Object]
impl DocumentMutation {
    /// Téléverse un document médical (patient ou médecin)
    async fn upload_document(
        &self,
        ctx: &Context<'_>,
        data: CreateDocumentInput,
    ) -> async_graphql::Result<Document> {
        let claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        let auth_uid = claims.supabase_uid().map_err(|_| "Invalid auth user ID")?;

        let profile = Profile::find_by_auth_uid(pool, auth_uid)
            .await?
            .ok_or("Profile not found. Call syncProfile first.")?;

        let doc = Document::create(pool, profile.id, data).await?;
        Ok(doc)
    }

    /// Supprime un document médical par id
    async fn delete_document(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let _claims = get_claims(ctx)?;
        let pool = ctx.data::<PgPool>()?;
        Document::delete(pool, id).await?;
        Ok(true)
    }
}
