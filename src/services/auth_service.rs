use crate::{error::AuthError, models::user::UserModel, repository::user_repository};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use sqlx::PgPool;
use validator::ValidateEmail;

// Minimum password length
const MIN_PASSWORD_LENGTH: usize = 8;

// Email validation using validator trait
fn is_valid_email(email: &str) -> bool {
    email.validate_email()
}

// Password validation
fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AuthError::WeakPassword(format!(
            "Password must be at least {} characters",
            MIN_PASSWORD_LENGTH
        )));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashingError(e.to_string()))
}

//Main logic register
pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<UserModel, AuthError> {
    // Validate email format
    if !is_valid_email(email) {
        return Err(AuthError::InvalidEmail);
    }

    // Validate password strength
    validate_password(password)?;

    // Check if user exists, handling potential DB errors explicitly
    match user_repository::find_user_by_email(pool, email).await {
        Ok(Some(_)) => return Err(AuthError::EmailAlreadyExists),
        Ok(None) => {}
        Err(e) => return Err(AuthError::DatabaseError(e.to_string())),
    }

    // Hash password
    let hashed_password = hash_password(password)?;

    // Create user - also handle duplicate key error here as backup
    let result = user_repository::create_user(pool, email, &hashed_password)
        .await
        .map_err(|e| {
            // Check if it's a unique constraint violation
            if e.to_string().contains("duplicate key")
                || e.to_string().contains("unique constraint")
            {
                AuthError::EmailAlreadyExists
            } else {
                AuthError::DatabaseError(e.to_string())
            }
        })?;

    Ok(result)
}
