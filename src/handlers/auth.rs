use crate::services::auth_service;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

pub async fn register_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match auth_service::register_user(&pool, &payload.email, &payload.password).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}
