use crate::config::R2Config;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};

/// Creates an S3 client configured for Cloudflare R2
pub async fn get_r2_client(config: &R2Config) -> Client {
    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", config.account_id);

    let credentials = Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "r2",
    );

    let s3_config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .region(Region::new("auto"))
        .endpoint_url(&endpoint_url)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}
