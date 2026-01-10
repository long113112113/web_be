use crate::models::friend::FriendshipModel;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub async fn create_request(
    pool: &PgPool,
    user_id: Uuid,
    friend_id: Uuid,
) -> Result<FriendshipModel, Error> {
    sqlx::query_as::<_, FriendshipModel>(
        "INSERT INTO friendships (user_id, friend_id, status) VALUES ($1, $2, 'pending') 
        RETURNING id, user_id, friend_id, status, created_at, updated_at",
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(pool)
    .await
}

pub async fn find_friendship(
    pool: &PgPool,
    user_id: Uuid,
    friend_id: Uuid,
) -> Result<Option<FriendshipModel>, Error> {
    sqlx::query_as::<_, FriendshipModel>(
        "SELECT * FROM friendships WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_optional(pool)
    .await
}

pub async fn accept_request(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<FriendshipModel>, Error> {
    sqlx::query_as::<_, FriendshipModel>(
        "UPDATE friendships SET status = 'accepted' WHERE id = $1 RETURNING id, user_id, friend_id, status, created_at, updated_at",
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_friendship(
    pool: &PgPool,
    user_id: Uuid,
    friend_id: Uuid,
) -> Result<u64, Error> {
    let result = sqlx::query(
        "DELETE FROM friendships WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// Struct to hold combined data for list views
#[derive(sqlx::FromRow)]
pub struct FriendWithProfile {
    pub user_id: Uuid,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_friends(pool: &PgPool, user_id: Uuid) -> Result<Vec<FriendWithProfile>, Error> {
    sqlx::query_as::<_, FriendWithProfile>(
        r#"
        SELECT 
            p.user_id, p.full_name, p.avatar_url, f.status, f.created_at
        FROM friendships f
        JOIN profiles p ON (f.user_id = p.user_id OR f.friend_id = p.user_id)
        WHERE (f.user_id = $1 OR f.friend_id = $1)
          AND f.status = 'accepted'
          AND p.user_id != $1
        ORDER BY p.full_name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_pending_requests(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<FriendWithProfile>, Error> {
    // Requests received by user_id
    sqlx::query_as::<_, FriendWithProfile>(
        r#"
        SELECT 
            p.user_id, p.full_name, p.avatar_url, f.status, f.created_at
        FROM friendships f
        JOIN profiles p ON f.user_id = p.user_id 
        WHERE f.friend_id = $1 AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_sent_requests(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<FriendWithProfile>, Error> {
    // Requests sent by user_id
    sqlx::query_as::<_, FriendWithProfile>(
        r#"
        SELECT 
            p.user_id, p.full_name, p.avatar_url, f.status, f.created_at
        FROM friendships f
        JOIN profiles p ON f.friend_id = p.user_id
        WHERE f.user_id = $1 AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
