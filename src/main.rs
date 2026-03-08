mod app;
mod appointments;
mod doctors;
mod hydration;
mod middlewares;
mod moods;
mod patients;
mod profiles;
mod utils;
mod weights;

use app::server::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let server = Server::new().await?;
    server.run().await?;
    Ok(())
}
