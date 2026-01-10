use crate::{error::AppError, models::friend::FriendshipModel, repository::friend_repository};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn request_friend(
    pool: &PgPool,
    user_id: Uuid,
    target_id: Uuid,
) -> Result<FriendshipModel, AppError> {
    if user_id == target_id {
        return Err(AppError::BadRequest("Cannot add yourself".into()));
    }

    // Check existing
    let existing = friend_repository::find_friendship(pool, user_id, target_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?;

    if let Some(_) = existing {
        return Err(AppError::BadRequest(
            "Friendship or request already exists".into(),
        ));
    }

    friend_repository::create_request(pool, user_id, target_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))
}

pub async fn accept_friend(
    pool: &PgPool,
    user_id: Uuid,
    target_id: Uuid,
) -> Result<FriendshipModel, AppError> {
    // We need to find the specific request where user_id is the *recipient* (friend_id in DB)
    // and target_id is the *sender* (user_id in DB).
    // Or we find the friendship record and verify.

    let existing = friend_repository::find_friendship(pool, user_id, target_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?
        .ok_or(AppError::BadRequest("Friend request not found".into()))?;

    if existing.status == "accepted" {
        return Err(AppError::BadRequest("Already friends".into()));
    }

    // Determine if user_id is the recipient
    if existing.friend_id != user_id {
        return Err(AppError::BadRequest(
            "You can only accept requests sent to you".into(),
        ));
    }

    friend_repository::accept_request(pool, existing.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?
        .ok_or(AppError::InternalError(
            "Failed to update friendship".into(),
        ))
}

pub async fn remove_friend_or_request(
    pool: &PgPool,
    user_id: Uuid,
    target_id: Uuid,
) -> Result<(), AppError> {
    let count = friend_repository::delete_friendship(pool, user_id, target_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string().into()))?;

    if count == 0 {
        return Err(AppError::BadRequest("Friendship not found".into()));
    }
    Ok(())
}
