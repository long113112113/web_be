use std::env;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    EnvVarMissing(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvVarMissing(var) => write!(f, "Environment variable {} must be set", var),
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn init() -> Result<Config, ConfigError> {
        let database_url = env::var("DATABASE_URL").map_err(|_| ConfigError::EnvVarMissing("DATABASE_URL".to_string()))?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::EnvVarMissing("JWT_SECRET".to_string()))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            database_url,
            jwt_secret,
            cors_origins,
        })
    }
}
