use crate::config::Config;
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub s3_client: S3Client,
}
