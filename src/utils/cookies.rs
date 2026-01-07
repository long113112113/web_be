use crate::constant::auth::REFRESH_TOKEN_DURATION_DAYS;
use axum_extra::extract::cookie::{Cookie, SameSite};

pub fn create_auth_cookies(access_token: String, refresh_token: String) -> Vec<Cookie<'static>> {
    let mut cookies = Vec::new();

    // Access Token Cookie
    // Expires in 1 hour (same as JWT)
    let access_cookie = Cookie::build(("access_token", access_token))
        .http_only(true)
        .secure(true) // Set to false if not running on HTTPS locally, but true is recommended
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.push(access_cookie);

    // Refresh Token Cookie
    // Expires in 7 days
    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(REFRESH_TOKEN_DURATION_DAYS))
        .build();
    cookies.push(refresh_cookie);

    cookies
}

pub fn remove_auth_cookies() -> Vec<Cookie<'static>> {
    let mut cookies = Vec::new();

    let access_cookie = Cookie::build(("access_token", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    cookies.push(access_cookie);

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    cookies.push(refresh_cookie);

    cookies
}
