use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use web_be::{config::Config, routes::auth_routes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let config = Config::init();

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    println!("Migrations executed successfully!");

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .cors_origins
                .iter()
                .map(|s| s.parse().expect("Invalid header value"))
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Setup Axum router
    let app = Router::new()
        .nest("/api/auth", auth_routes(pool.clone()))
        .layer(cors);

    // Initialize and start scheduler
    let sched = web_be::services::scheduler::init_scheduler(pool)
        .await
        .expect("Failed to initialize scheduler");
    sched.start().await.expect("Failed to start scheduler");

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
