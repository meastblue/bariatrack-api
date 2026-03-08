mod app;
mod appointments;
mod doctor_patients;
mod doctors;
mod hydration;
mod middlewares;
mod moods;
mod patients;
mod profiles;
mod utils;
mod supplements;
mod weights;
mod diet_phases;
mod wellness_tasks;
mod documents;
mod notifications;

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
