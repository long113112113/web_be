use crate::{
    constant::auth::REFRESH_TOKEN_COOKIE_NAME,
    dtos::private::auth::{
        request::{LoginRequest, RegisterRequest},
        response::AuthResponse,
    },
    error::AuthError,
    services::auth::auth_service,
    state::AppState,
    utils::cookies::{create_auth_cookies, remove_auth_cookies},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::CookieJar;
use validator::Validate;

/// Helper function to format validation errors into a readable message
fn format_validation_errors(errors: validator::ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |e| {
                e.message
                    .clone()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid {}", field))
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub async fn register_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate request using validator derive macros
    if let Err(errors) = payload.validate() {
        return AuthError::ValidationError(format_validation_errors(errors)).into_response();
    }

    match auth_service::register_user(
        &state.pool,
        payload.email.trim(),
        payload.password.trim(),
        &state.config.jwt_secret,
    )
    .await
    {
        Ok((token, refresh_token, user)) => {
            let cookies = create_auth_cookies(token, refresh_token, true);
            let mut updated_jar = jar;
            for cookie in cookies {
                updated_jar = updated_jar.add(cookie);
            }
            (
                StatusCode::CREATED,
                updated_jar,
                Json(AuthResponse { user }),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate request using validator derive macros
    if let Err(errors) = payload.validate() {
        return AuthError::ValidationError(format_validation_errors(errors)).into_response();
    }

    match auth_service::login_user(
        &state.pool,
        payload.email.trim(),
        payload.password.trim(),
        &state.config.jwt_secret,
    )
    .await
    {
        Ok((token, refresh_token, user)) => {
            let cookies = create_auth_cookies(token, refresh_token, payload.remember_me);
            let mut updated_jar = jar;
            for cookie in cookies {
                updated_jar = updated_jar.add(cookie);
            }
            (StatusCode::OK, updated_jar, Json(AuthResponse { user })).into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let refresh_token = match jar.get(REFRESH_TOKEN_COOKIE_NAME) {
        Some(cookie) => cookie.value(),
        None => return (StatusCode::UNAUTHORIZED, "Refresh token not found").into_response(),
    };

    match auth_service::refresh_access_token(&state.pool, refresh_token, &state.config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => {
            let cookies = create_auth_cookies(token, refresh_token, true);
            let mut updated_jar = jar;
            for cookie in cookies {
                updated_jar = updated_jar.add(cookie);
            }
            (StatusCode::OK, updated_jar, Json(AuthResponse { user })).into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn logout_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Invalidate refresh token in database if present
    if let Some(refresh_cookie) = jar.get(REFRESH_TOKEN_COOKIE_NAME) {
        let _ = auth_service::invalidate_refresh_token(&state.pool, refresh_cookie.value()).await;
    }

    let cookies = remove_auth_cookies();
    let mut updated_jar = jar;
    for cookie in cookies {
        updated_jar = updated_jar.add(cookie);
    }
    (StatusCode::OK, updated_jar, "Logged out").into_response()
}
