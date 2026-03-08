use async_graphql::{EmptySubscription, MergedObject, Schema};
use sqlx::PgPool;

use crate::appointments::schema::{AppointmentMutation, AppointmentQuery};
use crate::doctor_patients::schema::{DoctorPatientMutation, DoctorPatientQuery};
use crate::doctors::schema::{DoctorMutation, DoctorQuery};
use crate::hydration::schema::{HydrationMutation, HydrationQuery};
use crate::moods::schema::{MoodMutation, MoodQuery};
use crate::patients::schema::{PatientMutation, PatientQuery};
use crate::profiles::schema::{ProfileMutation, ProfileQuery};
use crate::weights::schema::{WeightMutation, WeightQuery};

// ── Merged Root Types ──────────────────────

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    pub HealthQuery,
    pub ProfileQuery,
    pub PatientQuery,
    pub DoctorQuery,
    pub DoctorPatientQuery,
    pub WeightQuery,
    pub MoodQuery,
    pub HydrationQuery,
    pub AppointmentQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    pub HealthMutation,
    pub ProfileMutation,
    pub PatientMutation,
    pub DoctorMutation,
    pub DoctorPatientMutation,
    pub WeightMutation,
    pub MoodMutation,
    pub HydrationMutation,
    pub AppointmentMutation,
);

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

pub fn build_schema(pool: &PgPool, supabase_url: String, supabase_service_key: String) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(pool.clone())
    .data(crate::app::routes::SupabaseAdmin { url: supabase_url, service_key: supabase_service_key })
    .finish()
}
