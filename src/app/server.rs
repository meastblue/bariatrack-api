use super::config::Config;
use super::routes::config_routes;
use crate::middlewares::cors::cors_middleware;
use crate::utils::auth::fetch_jwks_key;
use jsonwebtoken::DecodingKey;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub struct Server {
    pool: PgPool,
    config: Config,
    decoding_key: DecodingKey,
}

impl Server {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        info!("Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&config.db_url())
            .await
            .map_err(|e| format!("Database connection failed: {e}"))?;
        info!("Database connected");

        info!("Running migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| format!("Migration failed: {e}"))?;
        info!("Migrations applied");

        info!("Fetching Supabase JWKS...");
        let decoding_key = fetch_jwks_key(&config.supabase_url)
            .await
            .map_err(|e| format!("JWKS fetch failed: {e}"))?;
        info!("✅ JWKS loaded (ES256)");

        Ok(Self {
            pool,
            config,
            decoding_key,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = config_routes(&self.pool, self.decoding_key.clone()).layer(cors_middleware());
        let addr: SocketAddr =
            format!("{}:{}", self.config.server_host, self.config.server_port).parse()?;
        let listener = TcpListener::bind(addr).await?;

        info!("🚀 Server running at http://{addr}");
        info!("📊 GraphQL at http://{addr}/graphql");

        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        info!("Server shut down gracefully");
        Ok(())
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received");
}
