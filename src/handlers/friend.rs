use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    dtos::friend::FriendResponseDto, error::AppError, repository::friend_repository,
    services::friend_service, state::AppState, utils::jwt::Claims,
};

// Helper to convert repo result to DTO
fn map_to_dto(f: crate::repository::friend_repository::FriendWithProfile) -> FriendResponseDto {
    FriendResponseDto {
        user_id: f.user_id,
        full_name: f.full_name,
        avatar_url: f.avatar_url,
        status: f.status,
        created_at: f.created_at,
    }
}

pub async fn send_request_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    friend_service::request_friend(&state.pool, user_id, target_id).await?;

    Ok((StatusCode::CREATED, Json("Friend request sent")))
}

pub async fn accept_request_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    friend_service::accept_friend(&state.pool, user_id, target_id).await?;

    Ok(Json("Friend request accepted"))
}

pub async fn delete_friend_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(target_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    friend_service::remove_friend_or_request(&state.pool, user_id, target_id).await?;

    Ok(Json("Friendship/Request removed"))
}

pub async fn get_friends_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    let friends = friend_repository::get_friends(&state.pool, user_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?;

    let response: Vec<FriendResponseDto> = friends.into_iter().map(map_to_dto).collect();

    Ok(Json(response))
}

pub async fn get_pending_requests_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    let requests = friend_repository::get_pending_requests(&state.pool, user_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?;

    let response: Vec<FriendResponseDto> = requests.into_iter().map(map_to_dto).collect();

    Ok(Json(response))
}

pub async fn get_sent_requests_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user_id =
        Uuid::from_str(&claims.sub).map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    let requests = friend_repository::get_sent_requests(&state.pool, user_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?;

    let response: Vec<FriendResponseDto> = requests.into_iter().map(map_to_dto).collect();

    Ok(Json(response))
}
