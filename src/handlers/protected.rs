use crate::utils::jwt::Claims;
use axum::{Extension, Json, response::IntoResponse};

pub async fn me_handler(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    Json(claims)
}
