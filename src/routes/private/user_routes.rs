use crate::constant::image::MAX_AVATAR_SIZE;
use crate::handlers::profile::{edit_profile_handler, me_handler, upload_avatar_handler};
use crate::state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post, put},
};

pub fn user_routes(state: AppState) -> Router {
    // Routes without rate limiting
    let non_limited = Router::new().route("/me", get(me_handler));

    // Routes with rate limiting for upload protection
    // Uses shared config from AppState (per docs: do not create config multiple times!)
    let rate_limited = Router::new()
        .route("/avatar", post(upload_avatar_handler))
        .route("/edit", put(edit_profile_handler))
        .layer(DefaultBodyLimit::max(MAX_AVATAR_SIZE + 1024)) // Prevent DoS: limit body size before reading
        .layer(tower_governor::GovernorLayer::new(
            state.rate_limit_config.clone(),
        ));

    Router::new()
        .merge(non_limited)
        .merge(rate_limited)
        .with_state(state)
}
