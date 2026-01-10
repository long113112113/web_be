use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    constant::image::{ALLOWED_CONTENT_TYPES, MAX_AVATAR_SIZE},
    error::AppError,
    repository::profile_repository,
    services::profile_service,
    state::AppState,
    utils::{image::strip_metadata, jwt::Claims},
};

/// Helper function to process and upload avatar image
/// Validates file type/size, strips metadata, uploads to R2, and returns the URL
async fn process_and_upload_avatar(
    state: &AppState,
    user_id: Uuid,
    file_bytes: Vec<u8>,
    content_type: &str,
) -> Result<String, AppError> {
    // Validate file type
    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        return Err(AppError::BadRequest(
            "Invalid file type. Allowed: JPEG, PNG, GIF, WebP".into(),
        ));
    }

    // Validate file size
    if file_bytes.len() > MAX_AVATAR_SIZE {
        return Err(AppError::BadRequest(
            "File too large. Maximum size is 5MB".into(),
        ));
    }

    // Strip EXIF/metadata from image for privacy
    // Use spawn_blocking because image processing is CPU-intensive
    let ct = content_type.to_string();
    let cleaned_bytes =
        match tokio::task::spawn_blocking(move || strip_metadata(&file_bytes, &ct)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                tracing::error!("Failed to strip metadata: {:?}", e);
                return Err(AppError::BadRequest("Failed to process image".into()));
            }
            Err(e) => {
                tracing::error!("Task join error: {:?}", e);
                return Err(AppError::InternalError("Internal error".into()));
            }
        };

    // Upload to R2
    let avatar_url = profile_service::upload_avatar(
        &state.s3_client,
        &state.config.r2.bucket_name,
        &state.config.r2.public_url,
        user_id,
        cleaned_bytes,
        content_type,
    )
    .await
    .map_err(|e| {
        tracing::error!("Avatar upload failed: {:?}", e);
        AppError::InternalError("Failed to upload avatar".into())
    })?;

    Ok(avatar_url)
}

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
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or_default() == "avatar" {
            content_type = field.content_type().map(|s| s.to_string());
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read file".into()))?
                    .to_vec(),
            );
            break;
        }
    }

    let file_bytes = file_bytes.ok_or(AppError::BadRequest("No avatar file provided".into()))?;
    let content_type =
        content_type.ok_or(AppError::BadRequest("Content type not specified".into()))?;

    profile_repository::ensure_profile_exists(&state.pool, user_id)
        .await
        .map_err(|_| AppError::InternalError("Failed to ensure profile".into()))?;

    // Process and upload avatar using helper function
    let avatar_url = process_and_upload_avatar(&state, user_id, file_bytes, &content_type).await?;

    profile_repository::update_avatar_url(&state.pool, user_id, &avatar_url)
        .await
        .map_err(|_| AppError::InternalError("Failed to update profile".into()))?;

    Ok((StatusCode::OK, Json(AvatarResponse { avatar_url })))
}

use crate::{dtos::private::user::UserMeResponse, repository::user_repository};
use std::str::FromStr;

pub async fn me_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::BadRequest("Invalid user ID".into()))?;

    // Fetch user and profile in a single query (optimized, fixes N+1 query issue)
    let (user, profile_opt) = user_repository::find_user_with_profile(&state.pool, user_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?
        .ok_or_else(|| AppError::BadRequest("User not found".into()))?;

    // If profile doesn't exist, create it (rare case for new users)
    let profile = match profile_opt {
        Some(p) => p,
        None => profile_repository::ensure_profile_exists(&state.pool, user_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string().into()))?,
    };

    let response = UserMeResponse {
        id: user.id,
        email: user.email,
        role: user.role,
        full_name: profile.full_name,
        bio: profile.bio,
        avatar_url: profile.avatar_url,
        created_at: profile.created_at,
        updated_at: profile.updated_at,
    };

    Ok(Json(response))
}

use crate::{
    dtos::private::user::UpdateProfileResponse,
    utils::validation::{validate_bio, validate_full_name},
};

pub async fn edit_profile_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    let mut full_name: Option<String> = None;
    let mut bio: Option<String> = None;
    let mut avatar_bytes: Option<Vec<u8>> = None;
    let mut avatar_content_type: Option<String> = None;

    // Parse multipart form data
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or_default();

        match field_name {
            "full_name" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read full_name".into()))?;

                // Only set if non-empty
                if !text.is_empty() {
                    // Validate full_name
                    validate_full_name(&text).map_err(|e| AppError::BadRequest(e.into()))?;
                    full_name = Some(text);
                }
            }
            "bio" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read bio".into()))?;

                // Only set if non-empty
                if !text.is_empty() {
                    // Validate bio
                    validate_bio(&text).map_err(|e| AppError::BadRequest(e.into()))?;
                    bio = Some(text);
                }
            }
            "avatar" => {
                avatar_content_type = field.content_type().map(|s| s.to_string());
                avatar_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::BadRequest("Failed to read avatar file".into()))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    // Ensure profile exists
    profile_repository::ensure_profile_exists(&state.pool, user_id)
        .await
        .map_err(|_| AppError::InternalError("Failed to ensure profile".into()))?;

    let mut avatar_url: Option<String> = None;

    // Handle avatar upload if provided
    if let (Some(bytes), Some(content_type)) = (avatar_bytes, avatar_content_type) {
        // Process and upload avatar using helper function
        let url = process_and_upload_avatar(&state, user_id, bytes, &content_type).await?;
        avatar_url = Some(url);
    }

    // Update profile with provided fields
    let updated_profile = profile_repository::update_profile(
        &state.pool,
        user_id,
        full_name.as_deref(),
        bio.as_deref(),
        avatar_url.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update profile: {:?}", e);
        AppError::InternalError("Failed to update profile".into())
    })?;

    let response = UpdateProfileResponse {
        full_name: updated_profile.full_name,
        bio: updated_profile.bio,
        avatar_url: updated_profile.avatar_url,
        updated_at: updated_profile.updated_at,
    };

    Ok(Json(response))
}
