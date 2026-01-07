use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
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
) -> impl IntoResponse {
    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid user ID").into_response(),
    };

    // Extract file from multipart
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "avatar" {
            // Get content type
            content_type = field.content_type().map(|s| s.to_string());

            // Read bytes
            match field.bytes().await {
                Ok(bytes) => file_bytes = Some(bytes.to_vec()),
                Err(_) => {
                    return (StatusCode::BAD_REQUEST, "Failed to read file").into_response();
                }
            }
            break;
        }
    }

    // Validate presence
    let file_bytes = match file_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "No avatar file provided").into_response(),
    };

    let content_type = match content_type {
        Some(ct) => ct,
        None => return (StatusCode::BAD_REQUEST, "Content type not specified").into_response(),
    };

    // Validate content type
    if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid file type. Allowed: JPEG, PNG, GIF, WebP",
        )
            .into_response();
    }

    // Validate size
    if file_bytes.len() > MAX_AVATAR_SIZE {
        return (
            StatusCode::BAD_REQUEST,
            "File too large. Maximum size is 5MB",
        )
            .into_response();
    }

    // Ensure profile exists
    if let Err(_) = profile_repository::ensure_profile_exists(&state.pool, user_id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to ensure profile",
        )
            .into_response();
    }

    // Strip EXIF/metadata from image for privacy
    let cleaned_bytes = match strip_metadata(&file_bytes, &content_type) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to strip metadata: {:?}", e);
            return (StatusCode::BAD_REQUEST, "Failed to process image").into_response();
        }
    };

    // Upload to R2
    let avatar_url = match profile_service::upload_avatar(
        &state.s3_client,
        &state.config.r2.bucket_name,
        &state.config.r2.public_url,
        user_id,
        cleaned_bytes,
        &content_type,
    )
    .await
    {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("Avatar upload failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload avatar").into_response();
        }
    };

    // Update profile in database
    if let Err(_) = profile_repository::update_avatar_url(&state.pool, user_id, &avatar_url).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update profile",
        )
            .into_response();
    }

    (StatusCode::OK, Json(AvatarResponse { avatar_url })).into_response()
}
