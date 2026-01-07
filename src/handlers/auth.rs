use crate::{
    dtos::private::auth::{
        request::{LoginRequest, RefreshTokenRequest, RegisterRequest},
        response::{LoginResponse, RegisterResponse},
    },
    services::auth::auth_service,
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match auth_service::register_user(&state.pool, &payload.email, &payload.password, &state.config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::CREATED,
            Json(RegisterResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match auth_service::login_user(&state.pool, &payload.email, &payload.password, &state.config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::OK,
            Json(LoginResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    match auth_service::refresh_access_token(&state.pool, &payload.refresh_token, &state.config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::OK,
            Json(LoginResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
