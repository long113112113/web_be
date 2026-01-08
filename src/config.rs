use std::env;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    EnvVarMissing(String),
    InvalidConfig(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvVarMissing(var) => {
                write!(f, "Environment variable {} must be set", var)
            }
            ConfigError::InvalidConfig(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct R2Config {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
    pub public_url: String,
}

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub r2: R2Config,
}

impl Config {
    pub fn init() -> Result<Config, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::EnvVarMissing("DATABASE_URL".to_string()))?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::EnvVarMissing("JWT_SECRET".to_string()))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        // Security: Reject wildcard origins when credentials are enabled
        // Browsers block credentials + wildcard, but we validate at startup to fail fast
        for origin in &cors_origins {
            if origin == "*" {
                return Err(ConfigError::InvalidConfig(
                    "Wildcard CORS origin (*) is not allowed when credentials are enabled. \
                     Specify explicit origins instead."
                        .to_string(),
                ));
            }
        }

        let r2 = R2Config {
            account_id: env::var("R2_ACCOUNT_ID")
                .map_err(|_| ConfigError::EnvVarMissing("R2_ACCOUNT_ID".to_string()))?,
            access_key_id: env::var("R2_ACCESS_KEY_ID")
                .map_err(|_| ConfigError::EnvVarMissing("R2_ACCESS_KEY_ID".to_string()))?,
            secret_access_key: env::var("R2_SECRET_ACCESS_KEY")
                .map_err(|_| ConfigError::EnvVarMissing("R2_SECRET_ACCESS_KEY".to_string()))?,
            bucket_name: env::var("R2_BUCKET_NAME")
                .map_err(|_| ConfigError::EnvVarMissing("R2_BUCKET_NAME".to_string()))?,
            public_url: env::var("R2_PUBLIC_URL")
                .map_err(|_| ConfigError::EnvVarMissing("R2_PUBLIC_URL".to_string()))?,
        };

        Ok(Config {
            database_url,
            jwt_secret,
            cors_origins,
            r2,
        })
    }
}
