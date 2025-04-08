use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Comment model representing a YouTube comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// YouTube video ID
    pub video_id: String,
    
    /// Comment ID
    pub comment_id: String,
    
    /// Author name
    pub author: String,
    
    /// Comment text
    pub text: String,
    
    /// Number of likes
    pub like_count: i32,
    
    /// When the comment was published
    pub published_at: DateTime<Utc>,
    
    /// Replies to this comment
    pub replies: Vec<Reply>,
}

/// Reply model representing a reply to a YouTube comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    /// Reply ID
    pub reply_id: String,
    
    /// Author name
    pub author: String,
    
    /// Reply text
    pub text: String,
    
    /// Number of likes
    pub like_count: i32,
    
    /// When the reply was published
    pub published_at: DateTime<Utc>,
}
