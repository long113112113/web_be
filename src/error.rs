use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("{0}")]
    WeakPassword(String),
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Hashing error: {0}")]
    HashingError(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token creation error: {0}")]
    TokenCreationError(String),
    #[error("Invalid token type")]
    InvalidTokenType,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}

// Error response structure for JSON output
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Implement IntoResponse to automatically convert AuthError to HTTP responses
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AuthError::InvalidEmail => {
                (StatusCode::BAD_REQUEST, "Invalid email format".to_string())
            }
            AuthError::WeakPassword(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AuthError::EmailAlreadyExists => {
                (StatusCode::CONFLICT, "Email already exists".to_string())
            }
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            AuthError::InvalidTokenType => {
                (StatusCode::UNAUTHORIZED, "Invalid token type".to_string())
            }
            AuthError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AuthError::DatabaseError(_)
            | AuthError::HashingError(_)
            | AuthError::TokenCreationError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    InternalError(String),
}
