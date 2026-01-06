use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn init() -> Config {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Config {
            database_url,
            jwt_secret,
            cors_origins,
        }
    }
}
