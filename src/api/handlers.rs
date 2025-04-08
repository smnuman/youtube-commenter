use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::{db::Database, models::Comment, services::youtube};

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// Get comments for a YouTube video
pub async fn get_comments(
    Path(video_id): Path<String>,
    State(db): State<Database>,
) -> Result<Json<Vec<Comment>>, StatusCode> {
    info!("Fetching comments for video: {}", video_id);

    // First, try to get comments from the database
    match db.get_comments(&video_id).await {
        Ok(Some(comments)) => {
            info!("Found {} comments in database", comments.len());
            return Ok(Json(comments));
        }
        Ok(None) => {
            info!("No comments found in database, fetching from YouTube API");
        }
        Err(e) => {
            error!("Error fetching comments from database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // If not in database, fetch from YouTube API
    match youtube::fetch_comments(&video_id).await {
        Ok(comments) => {
            info!("Fetched {} comments from YouTube API", comments.len());
            
            // Save comments to database
            if let Err(e) = db.save_comments(&video_id, &comments).await {
                error!("Error saving comments to database: {}", e);
            }
            
            Ok(Json(comments))
        }
        Err(e) => {
            error!("Error fetching comments from YouTube API: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Save comments for a YouTube video
pub async fn save_comments(
    Path(video_id): Path<String>,
    State(db): State<Database>,
    Json(comments): Json<Vec<Comment>>,
) -> Result<StatusCode, StatusCode> {
    info!("Saving {} comments for video: {}", comments.len(), video_id);

    match db.save_comments(&video_id, &comments).await {
        Ok(_) => {
            info!("Successfully saved comments to database");
            Ok(StatusCode::CREATED)
        }
        Err(e) => {
            error!("Error saving comments to database: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
