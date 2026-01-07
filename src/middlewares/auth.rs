use crate::{
    state::AppState,
    utils::jwt::{TokenType, decode_jwt_with_type},
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;

pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let access_token = match jar.get("access_token") {
        Some(cookie) => cookie.value(),
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Missing access token").into_response());
        }
    };

    match decode_jwt_with_type(access_token, &state.config.jwt_secret, TokenType::Access) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid access token").into_response()),
    }
}
