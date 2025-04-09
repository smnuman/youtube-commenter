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
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::handlers::AppState;
use services::{auth::AuthService, youtube::YouTubeService, ai::AiService};

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
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db.clone())?);
    let youtube_service = Arc::new(YouTubeService::new(db.clone(), auth_service.clone()));
    let ai_service = Arc::new(AiService::new(db.clone()));
    
    // Initialize default AI models
    ai_service.init_default_models().await?;
    
    // Create application state
    let app_state = AppState {
        db: db.clone(),
        auth_service: auth_service.clone(),
        youtube_service: youtube_service.clone(),
        ai_service: ai_service.clone(),
    };

    // Build our application with routes
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/", get(|| async { "YouTube Commenter API" }))
        .route("/api/health", get(api::handlers::health_check))
        .route("/api/auth/url", get(api::handlers::get_auth_url))
        .route("/api/auth/callback", get(api::handlers::oauth_callback))
        .route("/api/videos", get(api::handlers::get_videos))
        .route("/api/comments/:video_id", get(api::handlers::get_comments))
        .route("/api/reply/generate", post(api::handlers::generate_reply))
        .route("/api/reply/post", post(api::handlers::post_reply))
        .route("/api/history", get(api::handlers::get_history))
        .layer(cors)
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
