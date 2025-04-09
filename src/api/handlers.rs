use axum::
    extract::{Path, State, Query, Json as AxumJson},
    http::{StatusCode, HeaderMap, Request},
    response::{IntoResponse, Response, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

use crate::db::Database;
use crate::models::{Comment, InteractionRecord, InteractionType, ai::ReplyGenerationRequest};
use crate::services::{auth::AuthService, youtube::YouTubeService, ai::AiService};

/// Application state
pub struct AppState {
    pub db: Database,
    pub auth_service: Arc<AuthService>,
    pub youtube_service: Arc<YouTubeService>,
    pub ai_service: Arc<AiService>,
}

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// Get comments for a YouTube video
pub async fn get_comments(
    Path(video_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Comment>>, StatusCode> {
    info!("Fetching comments for video: {}", video_id);
    
    // Get user ID from session
    let user_id = get_user_id_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // First, try to get comments from the database
    match state.db.get_comments(&video_id).await {
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
    match state.youtube_service.fetch_comments(&user_id, &video_id).await {
        Ok(comments) => {
            info!("Fetched {} comments from YouTube API", comments.len());
            Ok(Json(comments))
        }
        Err(e) => {
            error!("Error fetching comments from YouTube API: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get videos for the authenticated user
pub async fn get_videos(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::services::youtube::YouTubeVideo>>, StatusCode> {
    // Get user ID from session
    let user_id = get_user_id_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    
    match state.youtube_service.get_channel_videos(&user_id).await {
        Ok(videos) => {
            info!("Fetched {} videos for user", videos.len());
            Ok(Json(videos))
        }
        Err(e) => {
            error!("Error fetching videos: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate an AI reply to a comment
#[derive(Debug, Deserialize)]
pub struct GenerateReplyRequest {
    /// The comment ID to reply to
    pub comment_id: String,
    
    /// The tone to use for the reply
    #[serde(default = "default_tone")]
    pub tone: String,
    
    /// Additional instructions for the AI
    pub additional_instructions: Option<String>,
}

fn default_tone() -> String {
    "friendly".to_string()
}

#[derive(Debug, Serialize)]
pub struct GenerateReplyResponse {
    /// The generated reply text
    pub reply_text: String,
    
    /// The model used to generate the reply
    pub model: String,
}

pub async fn generate_reply(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumJson(request): AxumJson<GenerateReplyRequest>,
) -> Result<Json<GenerateReplyResponse>, StatusCode> {
    // Get user ID from session
    let user_id = get_user_id_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get the comment from the database
    let comment = match state.db.get_comment(&request.comment_id).await {
        Ok(Some(comment)) => comment,
        Ok(None) => {
            error!("Comment not found: {}", request.comment_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Error fetching comment: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get previous interactions with this commenter
    let previous_interactions = Vec::new(); // TODO: Implement this
    
    // Create AI request
    let ai_request = ReplyGenerationRequest {
        comment_text: comment.text.clone(),
        comment_author: comment.author.clone(),
        video_title: "YouTube Video".to_string(), // TODO: Get actual video title
        video_id: comment.video_id.clone(),
        previous_interactions,
        tone: request.tone,
        additional_instructions: request.additional_instructions,
        max_length: None,
        parameter_overrides: None,
    };
    
    // Generate reply
    match state.ai_service.generate_reply(&ai_request).await {
        Ok(response) => {
            // Record the interaction
            let interaction = InteractionRecord {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                video_id: comment.video_id.clone(),
                comment_id: comment.comment_id.clone(),
                reply_id: None,
                interaction_type: InteractionType::ReplyGenerated,
                timestamp: chrono::Utc::now(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("reply_text".to_string(), response.reply_text.clone());
                    data.insert("model".to_string(), response.model.clone());
                    data
                },
            };
            
            if let Err(e) = state.db.record_interaction(&interaction).await {
                error!("Error recording interaction: {}", e);
            }
            
            Ok(Json(GenerateReplyResponse {
                reply_text: response.reply_text,
                model: response.model,
            }))
        }
        Err(e) => {
            error!("Error generating reply: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Post a reply to a comment
#[derive(Debug, Deserialize)]
pub struct PostReplyRequest {
    /// The comment ID to reply to
    pub comment_id: String,
    
    /// The reply text
    pub reply_text: String,
    
    /// Whether this reply was generated by AI
    #[serde(default)]
    pub ai_generated: bool,
    
    /// The AI model used to generate this reply, if applicable
    pub ai_model: Option<String>,
}

pub async fn post_reply(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumJson(request): AxumJson<PostReplyRequest>,
) -> Result<Json<Reply>, StatusCode> {
    // Get user ID from session
    let user_id = get_user_id_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Post the reply to YouTube
    match state.youtube_service.post_reply(&user_id, &request.comment_id, &request.reply_text).await {
        Ok(mut reply) => {
            // Update AI-generated flag if needed
            if request.ai_generated {
                reply.ai_generated = true;
                reply.ai_model = request.ai_model;
            }
            
            // Record the interaction
            let interaction = InteractionRecord {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                video_id: "".to_string(), // We don't have the video ID here
                comment_id: request.comment_id.clone(),
                reply_id: Some(reply.reply_id.clone()),
                interaction_type: InteractionType::ReplyPosted,
                timestamp: chrono::Utc::now(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("reply_text".to_string(), reply.text.clone());
                    if let Some(model) = &reply.ai_model {
                        data.insert("ai_model".to_string(), model.clone());
                    }
                    data
                },
            };
            
            if let Err(e) = state.db.record_interaction(&interaction).await {
                error!("Error recording interaction: {}", e);
            }
            
            Ok(Json(reply))
        }
        Err(e) => {
            error!("Error posting reply: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get interaction history
pub async fn get_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<InteractionRecord>>, StatusCode> {
    // Get user ID from session
    let user_id = get_user_id_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get limit parameter
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100);
    
    match state.db.get_user_interactions(&user_id, limit).await {
        Ok(interactions) => {
            Ok(Json(interactions))
        }
        Err(e) => {
            error!("Error fetching interaction history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// OAuth callback handler
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    code: String,
    state: String,
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
    req: Request<axum::body::Body>,
) -> impl IntoResponse {
    // Exchange the code for tokens
    match state.auth_service.exchange_code(&params.code).await {
        Ok(token) => {
            // Create or update user
            match state.auth_service.create_or_update_user(&token).await {
                Ok(user) => {
                    // Create a session
                    let ip = req.remote_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let user_agent = req.headers()
                        .get("user-agent")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    match state.auth_service.create_session(&user.id, &ip, &user_agent).await {
                        Ok(session) => {
                            // Redirect to frontend with session ID
                            Redirect::to(&format!("/auth/success?session_id={}", session.id)).into_response()
                        }
                        Err(e) => {
                            error!("Error creating session: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Error creating user: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            error!("Error exchanging code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get authorization URL
pub async fn get_auth_url(
    State(state): State<AppState>,
) -> Json<Value> {
    let url = state.auth_service.get_authorization_url();
    Json(json!({ "url": url }))
}

/// Helper function to get user ID from headers
fn get_user_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers.get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Reply model for API responses
#[derive(Debug, Serialize)]
pub struct Reply {
    /// Reply ID
    pub reply_id: String,
    
    /// Parent comment ID
    pub parent_id: String,
    
    /// Author name
    pub author: String,
    
    /// Reply text
    pub text: String,
    
    /// Whether this reply was generated by AI
    pub ai_generated: bool,
    
    /// The AI model used to generate this reply, if applicable
    pub ai_model: Option<String>,
}
