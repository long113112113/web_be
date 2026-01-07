use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use web_be::{
    config::Config,
    routes::{private_routes, public_routes},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Initialize configuration with error handling
    let config = match Config::init() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };
    let config_arc = Arc::new(config);

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config_arc.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    println!("Migrations executed successfully!");

    // Setup CORS
    let allowed_origins = config_arc
        .cors_origins
        .iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_credentials(true);

    let app_state = AppState {
        pool: pool.clone(),
        config: config_arc.clone(),
    };

    // Setup Axum router
    let app = Router::new()
        .nest("/api", public_routes(app_state.clone()))
        .nest("/api", private_routes(app_state.clone()))
        .layer(cors);

    // Initialize and start scheduler
    let sched = web_be::services::scheduler::init_scheduler(pool)
        .await
        .map_err(|e| format!("Failed to initialize scheduler: {}", e))?;

    sched
        .start()
        .await
        .map_err(|e| format!("Failed to start scheduler: {}", e))?;

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
