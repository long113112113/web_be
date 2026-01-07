use crate::utils::jwt::Claims;
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::cookie::CookieJar;

pub async fn me_handler(
    Extension(claims): Extension<Claims>,
    _jar: CookieJar, // Just to show we can access cookies too
) -> impl IntoResponse {
    Json(claims)
}
