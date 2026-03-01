use crate::app::schema::{build_schema, AppSchema};
use crate::middlewares::auth::auth_middleware;
use crate::utils::auth::Claims;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use jsonwebtoken::DecodingKey;
use sqlx::PgPool;

pub fn config_routes(pool: &PgPool, decoding_key: DecodingKey) -> Router {
    let schema = build_schema(pool);

    let public = Router::new()
        .route("/", get(|| async { "BariaTrack API 🏥" }))
        .route("/health", get(|| async { "ok" }))
        .route("/graphql", get(graphql_playground));

    let protected = Router::new()
        .route("/graphql", post(graphql_handler))
        .layer(middleware::from_fn(auth_middleware))
        .layer(Extension(decoding_key));

    public.merge(protected).layer(Extension(schema))
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    Extension(claims): Extension<Claims>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let request = req.into_inner().data(claims);
    schema.execute(request).await.into()
}
