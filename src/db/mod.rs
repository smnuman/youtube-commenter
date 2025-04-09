use anyhow::{Context, Result};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use tracing::info;

use crate::models::{Comment, InteractionRecord, auth::{User, Session, AuthToken}, ai::AiModelConfig};

pub type Database = Surreal<Db>;

/// Initialize the SurrealDB database
pub async fn init_db() -> Result<Database> {
    info!("Initializing SurrealDB");
    
    // Create an in-memory database
    let db = Surreal::new::<Mem>(()).await?;
    
    // Select a namespace and database
    db.use_ns("youtube_commenter").use_db("main").await?;
    
    // Create schema for comments
    db.query("DEFINE TABLE comments SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD video_id ON TABLE comments TYPE string;
        DEFINE FIELD comment_id ON TABLE comments TYPE string;
        DEFINE FIELD author ON TABLE comments TYPE string;
        DEFINE FIELD author_channel_id ON TABLE comments TYPE string;
        DEFINE FIELD text ON TABLE comments TYPE string;
        DEFINE FIELD like_count ON TABLE comments TYPE int;
        DEFINE FIELD published_at ON TABLE comments TYPE datetime;
        DEFINE FIELD replies ON TABLE comments TYPE array;
        DEFINE FIELD replied_to ON TABLE comments TYPE bool;
        DEFINE FIELD metadata ON TABLE comments TYPE object;
        DEFINE INDEX video_id_idx ON TABLE comments COLUMNS video_id;
        DEFINE INDEX comment_id_idx ON TABLE comments COLUMNS comment_id;
    "#).await?;
    
    // Create schema for users
    db.query("DEFINE TABLE users SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD id ON TABLE users TYPE string;
        DEFINE FIELD name ON TABLE users TYPE string;
        DEFINE FIELD email ON TABLE users TYPE string;
        DEFINE FIELD profile_picture_url ON TABLE users TYPE string;
        DEFINE FIELD created_at ON TABLE users TYPE datetime;
        DEFINE FIELD updated_at ON TABLE users TYPE datetime;
        DEFINE FIELD preferences ON TABLE users TYPE object;
        DEFINE FIELD metadata ON TABLE users TYPE object;
        DEFINE INDEX user_id_idx ON TABLE users COLUMNS id;
    "#).await?;
    
    // Create schema for auth tokens
    db.query("DEFINE TABLE auth_tokens SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD user_id ON TABLE auth_tokens TYPE string;
        DEFINE FIELD access_token ON TABLE auth_tokens TYPE string;
        DEFINE FIELD refresh_token ON TABLE auth_tokens TYPE string;
        DEFINE FIELD expires_at ON TABLE auth_tokens TYPE datetime;
        DEFINE FIELD token_type ON TABLE auth_tokens TYPE string;
        DEFINE FIELD scopes ON TABLE auth_tokens TYPE array;
        DEFINE INDEX auth_user_id_idx ON TABLE auth_tokens COLUMNS user_id;
    "#).await?;
    
    // Create schema for sessions
    db.query("DEFINE TABLE sessions SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD id ON TABLE sessions TYPE string;
        DEFINE FIELD user_id ON TABLE sessions TYPE string;
        DEFINE FIELD created_at ON TABLE sessions TYPE datetime;
        DEFINE FIELD expires_at ON TABLE sessions TYPE datetime;
        DEFINE FIELD ip_address ON TABLE sessions TYPE string;
        DEFINE FIELD user_agent ON TABLE sessions TYPE string;
        DEFINE FIELD is_active ON TABLE sessions TYPE bool;
        DEFINE INDEX session_id_idx ON TABLE sessions COLUMNS id;
        DEFINE INDEX session_user_id_idx ON TABLE sessions COLUMNS user_id;
    "#).await?;
    
    // Create schema for interaction history
    db.query("DEFINE TABLE interactions SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD id ON TABLE interactions TYPE string;
        DEFINE FIELD user_id ON TABLE interactions TYPE string;
        DEFINE FIELD video_id ON TABLE interactions TYPE string;
        DEFINE FIELD comment_id ON TABLE interactions TYPE string;
        DEFINE FIELD reply_id ON TABLE interactions TYPE string;
        DEFINE FIELD interaction_type ON TABLE interactions TYPE string;
        DEFINE FIELD timestamp ON TABLE interactions TYPE datetime;
        DEFINE FIELD data ON TABLE interactions TYPE object;
        DEFINE INDEX interaction_user_id_idx ON TABLE interactions COLUMNS user_id;
        DEFINE INDEX interaction_video_id_idx ON TABLE interactions COLUMNS video_id;
        DEFINE INDEX interaction_comment_id_idx ON TABLE interactions COLUMNS comment_id;
    "#).await?;
    
    // Create schema for AI models
    db.query("DEFINE TABLE ai_models SCHEMAFULL").await?;
    db.query(r#"
        DEFINE FIELD model_id ON TABLE ai_models TYPE string;
        DEFINE FIELD name ON TABLE ai_models TYPE string;
        DEFINE FIELD description ON TABLE ai_models TYPE string;
        DEFINE FIELD max_context_length ON TABLE ai_models TYPE int;
        DEFINE FIELD max_response_length ON TABLE ai_models TYPE int;
        DEFINE FIELD parameters ON TABLE ai_models TYPE object;
        DEFINE FIELD is_available ON TABLE ai_models TYPE bool;
        DEFINE FIELD metadata ON TABLE ai_models TYPE object;
        DEFINE INDEX ai_model_id_idx ON TABLE ai_models COLUMNS model_id;
    "#).await?;
    
    info!("SurrealDB initialized successfully");
    
    Ok(db)
}

impl Database {
    // Comment methods
    
    /// Get comments for a video from the database
    pub async fn get_comments(&self, video_id: &str) -> Result<Option<Vec<Comment>>> {
        let result = self
            .query("SELECT * FROM comments WHERE video_id = $video_id")
            .bind(("video_id", video_id))
            .await?;
        
        let comments: Option<Vec<Comment>> = result.take(0)?;
        Ok(comments)
    }
    
    /// Save comments for a video to the database
    pub async fn save_comments(&self, video_id: &str, comments: &[Comment]) -> Result<()> {
        for comment in comments {
            self.create("comments")
                .content(comment)
                .await
                .with_context(|| format!("Failed to save comment {}", comment.comment_id))?;
        }
        
        Ok(())
    }
    
    /// Get a specific comment by ID
    pub async fn get_comment(&self, comment_id: &str) -> Result<Option<Comment>> {
        let result = self
            .query("SELECT * FROM comments WHERE comment_id = $comment_id LIMIT 1")
            .bind(("comment_id", comment_id))
            .await?;
        
        let comment: Option<Comment> = result.take(0)?;
        Ok(comment)
    }
    
    /// Update a comment's replied_to status
    pub async fn mark_comment_replied(&self, comment_id: &str, replied: bool) -> Result<()> {
        self.query("UPDATE comments SET replied_to = $replied WHERE comment_id = $comment_id")
            .bind(("comment_id", comment_id))
            .bind(("replied", replied))
            .await?;
        
        Ok(())
    }
    
    // User methods
    
    /// Create or update a user
    pub async fn save_user(&self, user: &User) -> Result<()> {
        self.query("DELETE FROM users WHERE id = $id")
            .bind(("id", &user.id))
            .await?;
            
        self.create("users")
            .content(user)
            .await
            .with_context(|| format!("Failed to save user {}", user.id))?;
        
        Ok(())
    }
    
    /// Get a user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let result = self
            .query("SELECT * FROM users WHERE id = $user_id LIMIT 1")
            .bind(("user_id", user_id))
            .await?;
        
        let user: Option<User> = result.take(0)?;
        Ok(user)
    }
    
    // Auth token methods
    
    /// Save an auth token
    pub async fn save_auth_token(&self, user_id: &str, token: &AuthToken) -> Result<()> {
        // Delete existing tokens for this user
        self.query("DELETE FROM auth_tokens WHERE user_id = $user_id")
            .bind(("user_id", user_id))
            .await?;
        
        // Create a new token record
        self.create("auth_tokens")
            .content((
                "user_id", user_id,
                "access_token", &token.access_token,
                "refresh_token", &token.refresh_token,
                "expires_at", &token.expires_at,
                "token_type", &token.token_type,
                "scopes", &token.scopes,
            ))
            .await
            .with_context(|| format!("Failed to save auth token for user {}", user_id))?;
        
        Ok(())
    }
    
    /// Get auth token for a user
    pub async fn get_auth_token(&self, user_id: &str) -> Result<Option<AuthToken>> {
        let result = self
            .query("SELECT * FROM auth_tokens WHERE user_id = $user_id LIMIT 1")
            .bind(("user_id", user_id))
            .await?;
        
        let token: Option<AuthToken> = result.take(0)?;
        Ok(token)
    }
    
    // Session methods
    
    /// Create a new session
    pub async fn create_session(&self, session: &Session) -> Result<()> {
        self.create("sessions")
            .content(session)
            .await
            .with_context(|| format!("Failed to create session {}", session.id))?;
        
        Ok(())
    }
    
    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let result = self
            .query("SELECT * FROM sessions WHERE id = $session_id LIMIT 1")
            .bind(("session_id", session_id))
            .await?;
        
        let session: Option<Session> = result.take(0)?;
        Ok(session)
    }
    
    /// End a session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        self.query("UPDATE sessions SET is_active = false WHERE id = $session_id")
            .bind(("session_id", session_id))
            .await?;
        
        Ok(())
    }
    
    // Interaction history methods
    
    /// Record an interaction
    pub async fn record_interaction(&self, interaction: &InteractionRecord) -> Result<()> {
        self.create("interactions")
            .content(interaction)
            .await
            .with_context(|| format!("Failed to record interaction {}", interaction.id))?;
        
        Ok(())
    }
    
    /// Get interactions for a user
    pub async fn get_user_interactions(&self, user_id: &str, limit: usize) -> Result<Vec<InteractionRecord>> {
        let result = self
            .query("SELECT * FROM interactions WHERE user_id = $user_id ORDER BY timestamp DESC LIMIT $limit")
            .bind(("user_id", user_id))
            .bind(("limit", limit))
            .await?;
        
        let interactions: Vec<InteractionRecord> = result.take(0)?;
        Ok(interactions)
    }
    
    /// Get interactions for a comment
    pub async fn get_comment_interactions(&self, comment_id: &str) -> Result<Vec<InteractionRecord>> {
        let result = self
            .query("SELECT * FROM interactions WHERE comment_id = $comment_id ORDER BY timestamp ASC")
            .bind(("comment_id", comment_id))
            .await?;
        
        let interactions: Vec<InteractionRecord> = result.take(0)?;
        Ok(interactions)
    }
    
    // AI model methods
    
    /// Save an AI model configuration
    pub async fn save_ai_model(&self, model: &AiModelConfig) -> Result<()> {
        self.query("DELETE FROM ai_models WHERE model_id = $model_id")
            .bind(("model_id", &model.model_id))
            .await?;
            
        self.create("ai_models")
            .content(model)
            .await
            .with_context(|| format!("Failed to save AI model {}", model.model_id))?;
        
        Ok(())
    }
    
    /// Get all available AI models
    pub async fn get_available_ai_models(&self) -> Result<Vec<AiModelConfig>> {
        let result = self
            .query("SELECT * FROM ai_models WHERE is_available = true")
            .await?;
        
        let models: Vec<AiModelConfig> = result.take(0)?;
        Ok(models)
    }
    
    /// Get an AI model by ID
    pub async fn get_ai_model(&self, model_id: &str) -> Result<Option<AiModelConfig>> {
        let result = self
            .query("SELECT * FROM ai_models WHERE model_id = $model_id LIMIT 1")
            .bind(("model_id", model_id))
            .await?;
        
        let model: Option<AiModelConfig> = result.take(0)?;
        Ok(model)
    }
}
