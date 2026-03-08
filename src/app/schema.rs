use async_graphql::{EmptySubscription, MergedObject, Schema};
use sqlx::PgPool;

use crate::patients::schema::{MutationRoot as PatientMutation, QueryRoot as PatientQuery};
use crate::profiles::schema::{MutationRoot as ProfileMutation, QueryRoot as ProfileQuery};

// ── Merged Root Types ──────────────────────

#[derive(MergedObject, Default)]
pub struct QueryRoot(pub HealthQuery, pub ProfileQuery, pub PatientQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(pub HealthMutation, pub ProfileMutation, pub PatientMutation);

// ── Health (infra) ─────────────────────────

#[derive(Default)]
pub struct HealthQuery;

#[async_graphql::Object]
impl HealthQuery {
    async fn health(&self) -> &str {
        "ok"
    }
}

#[derive(Default)]
pub struct HealthMutation;

#[async_graphql::Object]
impl HealthMutation {
    async fn ping(&self) -> &str {
        "pong"
    }
}

// ── Schema Builder ─────────────────────────

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(pool: &PgPool) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(pool.clone())
    .finish()
}
