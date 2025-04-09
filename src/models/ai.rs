use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    /// Model identifier
    pub model_id: String,
    
    /// Model name for display
    pub name: String,
    
    /// Model description
    pub description: String,
    
    /// Maximum context length
    pub max_context_length: usize,
    
    /// Maximum response length
    pub max_response_length: usize,
    
    /// Model parameters
    pub parameters: AiModelParameters,
    
    /// Whether this model is currently available
    pub is_available: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// AI model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelParameters {
    /// Temperature for generation (0.0 to 1.0)
    pub temperature: f32,
    
    /// Top-p sampling
    pub top_p: f32,
    
    /// Frequency penalty
    pub frequency_penalty: f32,
    
    /// Presence penalty
    pub presence_penalty: f32,
    
    /// Maximum number of tokens to generate
    pub max_tokens: usize,
    
    /// Stop sequences
    pub stop: Vec<String>,
    
    /// Additional parameters
    pub additional: HashMap<String, serde_json::Value>,
}

/// AI reply generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyGenerationRequest {
    /// The comment to reply to
    pub comment_text: String,
    
    /// The comment author
    pub comment_author: String,
    
    /// The video title
    pub video_title: String,
    
    /// The video ID
    pub video_id: String,
    
    /// Previous interactions with this commenter, if any
    pub previous_interactions: Vec<String>,
    
    /// The tone to use for the reply
    pub tone: String,
    
    /// Additional instructions for the AI
    pub additional_instructions: Option<String>,
    
    /// Maximum length of the reply
    pub max_length: Option<usize>,
    
    /// Model parameters to override
    pub parameter_overrides: Option<HashMap<String, serde_json::Value>>,
}

/// AI reply generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyGenerationResponse {
    /// The generated reply text
    pub reply_text: String,
    
    /// Alternative replies
    pub alternatives: Vec<String>,
    
    /// The model used to generate the reply
    pub model: String,
    
    /// When the reply was generated
    pub generated_at: DateTime<Utc>,
    
    /// Metadata about the generation process
    pub metadata: HashMap<String, String>,
    
    /// Usage statistics
    pub usage: AiUsageStats,
}

/// AI usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageStats {
    /// Number of prompt tokens
    pub prompt_tokens: usize,
    
    /// Number of completion tokens
    pub completion_tokens: usize,
    
    /// Total tokens used
    pub total_tokens: usize,
    
    /// Time taken to generate the reply (in milliseconds)
    pub generation_time_ms: u64,
}
