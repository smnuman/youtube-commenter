use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User model representing a YouTube account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (YouTube channel ID)
    pub id: String,
    
    /// User's display name
    pub name: String,
    
    /// User's email address
    pub email: Option<String>,
    
    /// URL to user's profile picture
    pub profile_picture_url: Option<String>,
    
    /// When the user was created in our system
    pub created_at: DateTime<Utc>,
    
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
    
    /// User's preferences
    pub preferences: UserPreferences,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Whether to enable AI reply suggestions
    pub enable_ai_replies: bool,
    
    /// The AI model to use for generating replies
    pub ai_model: String,
    
    /// The tone to use for AI-generated replies
    pub reply_tone: ReplyTone,
    
    /// Whether to enable real-time notifications
    pub enable_notifications: bool,
    
    /// How frequently to check for new comments (in seconds)
    pub polling_interval: u32,
    
    /// Additional preferences
    pub additional: HashMap<String, String>,
}

/// Tone options for AI-generated replies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplyTone {
    /// Professional and formal
    Professional,
    
    /// Friendly and casual
    Friendly,
    
    /// Enthusiastic and excited
    Enthusiastic,
    
    /// Helpful and informative
    Helpful,
    
    /// Custom tone with specific instructions
    Custom(String),
}

/// Authentication token for YouTube API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The access token
    pub access_token: String,
    
    /// The refresh token
    pub refresh_token: String,
    
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    
    /// The token type (usually "Bearer")
    pub token_type: String,
    
    /// The scopes this token has access to
    pub scopes: Vec<String>,
}

/// Session information for a logged-in user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    
    /// User ID
    pub user_id: String,
    
    /// When the session was created
    pub created_at: DateTime<Utc>,
    
    /// When the session expires
    pub expires_at: DateTime<Utc>,
    
    /// IP address that created this session
    pub ip_address: String,
    
    /// User agent that created this session
    pub user_agent: String,
    
    /// Whether this session is currently active
    pub is_active: bool,
}
