use axum::Router;
use sqlx::postgres::PgPoolOptions;
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

    // Setup Axum router
    let app = Router::new().nest("/api/auth", auth_routes(pool));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
