use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use web_be::{
    config::Config,
    routes::{private_routes, public_routes},
    state::AppState,
    utils::s3::get_r2_client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Initialize configuration with error handling
    let config = Config::init()?;
    let config_arc = Arc::new(config);

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config_arc.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    // Warm up database connections
    let _ = sqlx::query("SELECT 1").fetch_one(&pool).await?;

    // Setup CORS
    let allowed_origins = config_arc
        .cors_origins
        .iter()
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    // Initialize R2/S3 client
    let s3_client = get_r2_client(&config_arc.r2).await;

    // Create rate limit config ONCE (per docs: do not create multiple times!)
    // Allow bursts with up to 5 requests per IP and replenishes at 1 request per second
    let rate_limit_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(tower_governor::key_extractor::SmartIpKeyExtractor)
            .per_second(1)
            .burst_size(5)
            .finish()
            .expect("Failed to build rate limit config"),
    );

    let app_state = AppState {
        pool: pool.clone(),
        config: config_arc.clone(),
        s3_client,
        rate_limit_config,
    };

    // Setup Axum router
    let app = Router::new()
        .nest(
            "/api",
            public_routes(app_state.clone()).merge(private_routes(app_state.clone())),
        )
        .layer(cors)
        //TODO: Remove logging layer in production
        .layer(TraceLayer::new_for_http());

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
    // Use into_make_service_with_connect_info for rate limiting IP extraction (per tower-governor docs)
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
