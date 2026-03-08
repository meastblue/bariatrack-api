use std::env;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load .env file")]
    DotenvError(dotenvy::Error),
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid port number: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub server_host: String,
    pub server_port: u16,
    pub supabase_url: String,
    pub supabase_service_key: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().map_err(ConfigError::DotenvError)?;

        let db_host =
            env::var("DB_HOST").map_err(|_| ConfigError::MissingEnvVar("DB_HOST".into()))?;
        let db_port = env::var("DB_PORT")
            .map_err(|_| ConfigError::MissingEnvVar("DB_PORT".into()))?
            .parse()?;
        let db_name =
            env::var("DB_NAME").map_err(|_| ConfigError::MissingEnvVar("DB_NAME".into()))?;
        let db_user =
            env::var("DB_USER").map_err(|_| ConfigError::MissingEnvVar("DB_USER".into()))?;
        let db_password =
            env::var("DB_PASSWORD").map_err(|_| ConfigError::MissingEnvVar("DB_PASSWORD".into()))?;
        let server_host = env::var("SERVER_HOST")
            .map_err(|_| ConfigError::MissingEnvVar("SERVER_HOST".into()))?;
        let server_port = env::var("SERVER_PORT")
            .map_err(|_| ConfigError::MissingEnvVar("SERVER_PORT".into()))?
            .parse()?;
        let supabase_url = env::var("SUPABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("SUPABASE_URL".into()))?;
        let supabase_service_key = env::var("SUPABASE_SECRET_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("SUPABASE_SECRET_KEY".into()))?;

        Ok(Config {
            db_host,
            db_port,
            db_name,
            db_user,
            db_password,
            server_host,
            server_port,
            supabase_url,
            supabase_service_key,
        })
    }

    pub fn db_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, self.db_password, self.db_host, self.db_port, self.db_name
        )
    }
}
