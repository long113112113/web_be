use crate::{
    repository::user_repository,
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
use uuid::Uuid;

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
            // Parse user_id from token claims
            let user_id = Uuid::parse_str(&claims.sub)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token").into_response())?;

            // Verify user exists and is not deleted
            let user = user_repository::find_user_by_id(&state.pool, user_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?
                .ok_or((StatusCode::UNAUTHORIZED, "User not found").into_response())?;

            if user.is_deleted {
                return Err((StatusCode::UNAUTHORIZED, "User account deleted").into_response());
            }

            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid access token").into_response()),
    }
}
