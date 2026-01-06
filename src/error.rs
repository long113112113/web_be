use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    InvalidEmail,
    WeakPassword(String),
    EmailAlreadyExists,
    DatabaseError(String),
    HashingError(String),
    InvalidCredentials,
    TokenCreationError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidEmail => write!(f, "Invalid email format"),
            AuthError::WeakPassword(msg) => write!(f, "{}", msg),
            AuthError::EmailAlreadyExists => write!(f, "Email already exists"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::HashingError(msg) => write!(f, "Hashing error: {}", msg),
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenCreationError(msg) => write!(f, "Token creation error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

// Error response structure for JSON output
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Implement IntoResponse to automatically convert AuthError to HTTP responses
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::InvalidEmail => (StatusCode::BAD_REQUEST, "Invalid email format"),
            AuthError::WeakPassword(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            AuthError::DatabaseError(_)
            | AuthError::HashingError(_)
            | AuthError::TokenCreationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: message.to_string(),
            }),
        )
            .into_response()
    }
}
