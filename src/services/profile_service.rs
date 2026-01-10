use crate::error::AppError;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

/// Uploads an avatar to R2 and returns the public URL.
/// File is renamed using UUID to avoid conflicts and ensure unique naming.
pub async fn upload_avatar(
    s3_client: &S3Client,
    bucket: &str,
    public_url: &str,
    user_id: Uuid,
    file_bytes: Vec<u8>,
    content_type: &str,
) -> Result<String, AppError> {
    // Generate unique filename using UUID
    let extension = match content_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    };

    let file_uuid = Uuid::new_v4();
    let key = format!("avatars/{}/{}.{}", user_id, file_uuid, extension);

    // Upload to R2
    s3_client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(ByteStream::from(file_bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to upload avatar: {}", e).into()))?;

    // Return public URL
    let avatar_url = format!("{}/{}", public_url.trim_end_matches('/'), key);
    Ok(avatar_url)
}
