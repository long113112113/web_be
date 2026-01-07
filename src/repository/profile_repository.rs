use crate::models::profile::ProfileModel;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ProfileModel>, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        "SELECT id, user_id, full_name, bio, avatar_url, created_at, updated_at FROM profiles WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_profile(pool: &PgPool, user_id: Uuid) -> Result<ProfileModel, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        "INSERT INTO profiles (user_id) VALUES ($1) RETURNING id, user_id, full_name, bio, avatar_url, created_at, updated_at"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn update_avatar_url(
    pool: &PgPool,
    user_id: Uuid,
    avatar_url: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE profiles SET avatar_url = $1 WHERE user_id = $2")
        .bind(avatar_url)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Ensures a profile exists for the user, creating one if needed
pub async fn ensure_profile_exists(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ProfileModel, sqlx::Error> {
    match find_by_user_id(pool, user_id).await? {
        Some(profile) => Ok(profile),
        None => create_profile(pool, user_id).await,
    }
}
