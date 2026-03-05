mod app;
mod middlewares;
mod utils;
mod profiles;

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
