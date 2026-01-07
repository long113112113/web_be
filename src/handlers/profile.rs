use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    repository::profile_repository,
    services::profile_service,
    state::AppState,
    utils::{image::strip_metadata, jwt::Claims},
};

const MAX_AVATAR_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_CONTENT_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];

#[derive(Serialize)]
pub struct AvatarResponse {
    pub avatar_url: String,
}

pub async fn upload_avatar_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or_default() == "avatar" {
            content_type = field.content_type().map(|s| s.to_string());
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read file".to_string()))?
                    .to_vec(),
            );
            break;
        }
    }

    let file_bytes =
        file_bytes.ok_or(AppError::BadRequest("No avatar file provided".to_string()))?;
    let content_type = content_type
        .ok_or(AppError::BadRequest("Content type not specified".to_string()))?;

    if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid file type. Allowed: JPEG, PNG, GIF, WebP".to_string(),
        ));
    }

    if file_bytes.len() > MAX_AVATAR_SIZE {
        return Err(AppError::BadRequest(
            "File too large. Maximum size is 5MB".to_string(),
        ));
    }

    profile_repository::ensure_profile_exists(&state.pool, user_id)
        .await
        .map_err(|_| AppError::InternalError("Failed to ensure profile".to_string()))?;

    // Strip EXIF/metadata from image for privacy
    // Use spawn_blocking because image processing is CPU-intensive
    let ct = content_type.clone();
    let cleaned_bytes =
        match tokio::task::spawn_blocking(move || strip_metadata(&file_bytes, &ct)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                tracing::error!("Failed to strip metadata: {:?}", e);
                return (StatusCode::BAD_REQUEST, "Failed to process image").into_response();
            }
            Err(e) => {
                tracing::error!("Task join error: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
            }
        };

    let avatar_url = profile_service::upload_avatar(
        &state.s3_client,
        &state.config.r2.bucket_name,
        &state.config.r2.public_url,
        user_id,
        cleaned_bytes,
        &content_type,
    )
    .await
    .map_err(|e| {
        tracing::error!("Avatar upload failed: {:?}", e);
        AppError::InternalError("Failed to upload avatar".to_string())
    })?;

    profile_repository::update_avatar_url(&state.pool, user_id, &avatar_url)
        .await
        .map_err(|_| AppError::InternalError("Failed to update profile".to_string()))?;

    Ok((StatusCode::OK, Json(AvatarResponse { avatar_url })))
}
