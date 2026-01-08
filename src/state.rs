use crate::config::Config;
use aws_sdk_s3::Client as S3Client;
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use sqlx::PgPool;
use std::sync::Arc;
use tower_governor::governor::GovernorConfig;
use tower_governor::key_extractor::SmartIpKeyExtractor;

/// Rate limit config type for sharing across requests
pub type RateLimitConfig = Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>>>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub s3_client: S3Client,
    /// Shared rate limit config (per docs: do not create config multiple times!)
    pub rate_limit_config: RateLimitConfig,
}
