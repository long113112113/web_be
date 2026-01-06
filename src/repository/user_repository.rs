use crate::models::user::UserModel;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserModel>, sqlx::Error> {
    let user = sqlx::query_as!(UserModel, "SELECT * FROM users_auth WHERE id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<UserModel, sqlx::Error> {
    let user = sqlx::query_as!(
        UserModel,
        "INSERT INTO users_auth (email, password_hash) VALUES ($1, $2) RETURNING *",
        email,
        password_hash
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserModel>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users_auth WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await?;
    Ok(user)
}
