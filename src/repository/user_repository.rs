use crate::models::profile::ProfileModel;
use crate::models::user::UserModel;
use sqlx::{PgPool, Row};
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

/// Fetches user with profile in a single query using LEFT JOIN
/// Returns None if user doesn't exist, returns user with None profile if profile doesn't exist
pub async fn find_user_with_profile(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<(UserModel, Option<ProfileModel>)>, sqlx::Error> {
    let result = sqlx::query(
        r#"
        SELECT 
            u.id as user_id, u.email, u.password_hash, u.role, u.is_active, u.is_deleted,
            u.created_at as user_created_at, u.updated_at as user_updated_at,
            p.id as profile_id, p.user_id as profile_user_id, p.full_name, p.bio, p.avatar_url, 
            p.created_at as profile_created_at, p.updated_at as profile_updated_at
        FROM users_auth u
        LEFT JOIN profiles p ON u.id = p.user_id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match result {
        None => Ok(None),
        Some(row) => {
            let user = UserModel {
                id: row.try_get("user_id")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                role: row.try_get("role")?,
                is_active: row.try_get("is_active")?,
                is_deleted: row.try_get("is_deleted")?,
                created_at: row.try_get("user_created_at")?,
                updated_at: row.try_get("user_updated_at")?,
            };

            let profile_id: Option<Uuid> = row.try_get("profile_id").ok();
            let profile = if profile_id.is_some() {
                Some(ProfileModel {
                    id: row.try_get("profile_id")?,
                    user_id: row.try_get("profile_user_id")?,
                    full_name: row.try_get("full_name")?,
                    bio: row.try_get("bio")?,
                    avatar_url: row.try_get("avatar_url")?,
                    created_at: row.try_get("profile_created_at")?,
                    updated_at: row.try_get("profile_updated_at")?,
                })
            } else {
                None
            };

            Ok(Some((user, profile)))
        }
    }
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
