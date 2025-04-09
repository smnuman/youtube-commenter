use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::Database;
use crate::models::ai::{AiModelConfig, AiModelParameters, ReplyGenerationRequest, ReplyGenerationResponse, AiUsageStats};
use crate::models::auth::{User, ReplyTone};

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// OpenAI API request
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiRequestMessage>,
    temperature: f32,
    max_tokens: usize,
    top_p: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
}

#[derive(Debug, Serialize)]
struct OpenAiRequestMessage {
    role: String,
    content: String,
}

/// AI service for generating replies
pub struct AiService {
    db: Database,
    client: Client,
}

impl AiService {
    /// Create a new AI service
    pub fn new(db: Database) -> Self {
        let client = Client::new();
        Self { db, client }
    }
    
    /// Initialize default AI models
    pub async fn init_default_models(&self) -> Result<()> {
        // GPT-3.5 Turbo
        let gpt35_turbo = AiModelConfig {
            model_id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            description: "A good balance of quality and speed for most reply generation needs".to_string(),
            max_context_length: 4096,
            max_response_length: 1024,
            parameters: AiModelParameters {
                temperature: 0.7,
                top_p: 1.0,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                max_tokens: 1024,
                stop: vec![],
                additional: Default::default(),
            },
            is_available: true,
            metadata: Default::default(),
        };
        
        // GPT-4
        let gpt4 = AiModelConfig {
            model_id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            description: "Highest quality replies with better understanding of context and nuance".to_string(),
            max_context_length: 8192,
            max_response_length: 2048,
            parameters: AiModelParameters {
                temperature: 0.7,
                top_p: 1.0,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                max_tokens: 2048,
                stop: vec![],
                additional: Default::default(),
            },
            is_available: true,
            metadata: Default::default(),
        };
        
        // Save models to database
        self.db.save_ai_model(&gpt35_turbo).await?;
        self.db.save_ai_model(&gpt4).await?;
        
        info!("Initialized default AI models");
        
        Ok(())
    }
    
    /// Generate a reply to a comment
    pub async fn generate_reply(&self, request: &ReplyGenerationRequest) -> Result<ReplyGenerationResponse> {
        // Get the AI model configuration
        let model_id = if let Some(override_model) = request.parameter_overrides.as_ref().and_then(|p| p.get("model")) {
            override_model.as_str().unwrap_or("gpt-3.5-turbo").to_string()
        } else {
            "gpt-3.5-turbo".to_string()
        };
        
        let model = match self.db.get_ai_model(&model_id).await? {
            Some(m) => m,
            None => anyhow::bail!("AI model {} not found", model_id),
        };
        
        // Build the prompt
        let system_message = self.build_system_message(&request.tone);
        let user_message = self.build_user_message(request);
        
        // Create OpenAI request
        let openai_request = OpenAiRequest {
            model: model.model_id.clone(),
            messages: vec![
                OpenAiRequestMessage {
                    role: "system".to_string(),
                    content: system_message,
                },
                OpenAiRequestMessage {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            temperature: model.parameters.temperature,
            max_tokens: request.max_length.unwrap_or(model.parameters.max_tokens),
            top_p: model.parameters.top_p,
            frequency_penalty: model.parameters.frequency_penalty,
            presence_penalty: model.parameters.presence_penalty,
        };
        
        // Get OpenAI API key
        let api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;
        
        // Send request to OpenAI
        let start_time = std::time::Instant::now();
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&openai_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OpenAI API error: {}", error_text);
            anyhow::bail!("Failed to generate reply: {}", error_text);
        }
        
        let openai_response: OpenAiResponse = response.json().await?;
        let generation_time = start_time.elapsed().as_millis() as u64;
        
        // Extract the generated reply
        let reply_text = openai_response.choices.get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        
        // Create response
        let response = ReplyGenerationResponse {
            reply_text,
            alternatives: vec![],
            model: model.model_id,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
            usage: AiUsageStats {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
                generation_time_ms: generation_time,
            },
        };
        
        Ok(response)
    }
    
    /// Build the system message for the AI
    fn build_system_message(&self, tone: &str) -> String {
        let base_instructions = "You are an assistant helping a YouTube content creator respond to comments on their videos. \
            Your goal is to write thoughtful, authentic replies that engage with the commenter and foster a positive community. \
            Keep replies concise, friendly, and conversational. Avoid generic responses.";
        
        let tone_instructions = match tone {
            "professional" => "Maintain a professional and informative tone. Be helpful and knowledgeable while remaining approachable.",
            "friendly" => "Be warm, casual, and conversational. Use a friendly tone as if chatting with someone you know well.",
            "enthusiastic" => "Be energetic and excited in your response. Show enthusiasm and appreciation for the commenter.",
            "helpful" => "Focus on being as helpful as possible. Provide useful information and address any questions thoroughly.",
            _ => "Use a balanced, friendly tone that's authentic and engaging.",
        };
        
        format!("{}\n\n{}", base_instructions, tone_instructions)
    }
    
    /// Build the user message containing the comment to reply to
    fn build_user_message(&self, request: &ReplyGenerationRequest) -> String {
        let mut message = format!(
            "Please write a reply to the following comment on my YouTube video titled \"{}\":\n\n",
            request.video_title
        );
        
        message.push_str(&format!("Comment from {}: \"{}\"\n\n", request.comment_author, request.comment_text));
        
        if !request.previous_interactions.is_empty() {
            message.push_str("Previous interactions with this commenter:\n");
            for interaction in &request.previous_interactions {
                message.push_str(&format!("- {}\n", interaction));
            }
            message.push('\n');
        }
        
        if let Some(instructions) = &request.additional_instructions {
            message.push_str(&format!("Additional instructions: {}\n\n", instructions));
        }
        
        message.push_str("Write only the reply text without any additional formatting or explanation.");
        
        message
    }
}
