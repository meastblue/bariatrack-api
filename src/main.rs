mod app;
mod doctors;
mod middlewares;
mod patients;
mod profiles;
mod utils;

use app::server::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let server = Server::new().await?;
    server.run().await?;
    Ok(())
}
