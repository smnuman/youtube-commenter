mod api;
mod db;
mod models;
mod services;
mod utils;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting YouTube Commenter API server");

    // Initialize database
    let db = db::init_db().await?;

    // Build our application with a route
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { "YouTube Commenter API" }))
        .route("/api/health", get(api::health_check))
        .route("/api/comments/:video_id", get(api::get_comments))
        .route("/api/comments/:video_id", post(api::save_comments))
        .layer(cors)
        .with_state(db);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
