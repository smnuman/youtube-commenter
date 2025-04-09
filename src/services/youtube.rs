use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, collections::HashMap, time::Duration};
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::Database;
use crate::models::{Comment, Reply, InteractionRecord, InteractionType};
use crate::services::auth::AuthService;

/// YouTube service for interacting with the YouTube API
pub struct YouTubeService {
    db: Database,
    client: Client,
    auth_service: AuthService,
}

impl YouTubeService {
    /// Create a new YouTube service
    pub fn new(db: Database, auth_service: AuthService) -> Self {
        let client = Client::new();
        Self { db, client, auth_service }
    }

    /// Fetch comments for a YouTube video
    pub async fn fetch_comments(&self, user_id: &str, video_id: &str) -> Result<Vec<Comment>> {
        info!("Fetching comments for video: {}", video_id);

        // Get a valid access token
        let access_token = self.auth_service.get_valid_access_token(user_id).await?;

        // Fetch comment threads
        let comment_threads = self.fetch_comment_threads(video_id, &access_token).await?;

        // Convert to our Comment model
        let mut comments = Vec::new();

        for thread in comment_threads {
            let comment_id = thread.id;
            let snippet = thread.snippet.top_level_comment.snippet;

            // Fetch replies if there are any
            let replies = if thread.snippet.total_reply_count > 0 {
                self.fetch_replies(&comment_id, &access_token).await?
            } else {
                Vec::new()
            };

            comments.push(Comment {
                video_id: video_id.to_string(),
                comment_id,
                author: snippet.author_display_name,
                author_channel_id: snippet.author_channel_id.value,
                text: snippet.text_display,
                like_count: snippet.like_count,
                published_at: snippet.published_at,
                replies,
                replied_to: false, // Will be updated from database
                metadata: HashMap::new(),
            });
        }

        info!("Fetched {} comments with replies", comments.len());

        // Update replied_to status from database
        for comment in &mut comments {
            if let Some(db_comment) = self.db.get_comment(&comment.comment_id).await? {
                comment.replied_to = db_comment.replied_to;
            }
        }

        // Save comments to database
        self.db.save_comments(video_id, &comments).await?;

        // Record interaction for each comment
        for comment in &comments {
            let interaction = InteractionRecord {
                id: Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                video_id: video_id.to_string(),
                comment_id: comment.comment_id.clone(),
                reply_id: None,
                interaction_type: InteractionType::CommentReceived,
                timestamp: Utc::now(),
                data: HashMap::new(),
            };

            self.db.record_interaction(&interaction).await?;
        }

        Ok(comments)
    }

    /// Fetch comment threads from YouTube API
    async fn fetch_comment_threads(
        &self,
        video_id: &str,
        access_token: &str,
    ) -> Result<Vec<YouTubeCommentThread>> {
        let mut all_threads = Vec::new();
        let mut page_token = None;

        loop {
            let url = format!(
                "https://www.googleapis.com/youtube/v3/commentThreads?part=snippet&videoId={}&maxResults=100{}",
                video_id,
                page_token.map_or(String::new(), |token| format!("&pageToken={}", token))
            );

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                error!("YouTube API error: {}", error_text);
                anyhow::bail!("YouTube API error: {}", error_text);
            }

            let response_data: YouTubeCommentThreadResponse = response.json().await?;

            all_threads.extend(response_data.items);

            // Check if there are more pages
            if let Some(token) = response_data.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        Ok(all_threads)
    }

    /// Fetch replies to a comment from YouTube API
    async fn fetch_replies(
        &self,
        comment_id: &str,
        access_token: &str,
    ) -> Result<Vec<Reply>> {
        let mut all_replies = Vec::new();
        let mut page_token = None;

        loop {
            let url = format!(
                "https://www.googleapis.com/youtube/v3/comments?part=snippet&parentId={}&maxResults=100{}",
                comment_id,
                page_token.map_or(String::new(), |token| format!("&pageToken={}", token))
            );

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                error!("YouTube API error: {}", error_text);
                anyhow::bail!("YouTube API error: {}", error_text);
            }

            let response_data: YouTubeCommentResponse = response.json().await?;

            let replies: Vec<Reply> = response_data
                .items
                .into_iter()
                .map(|item| Reply {
                    reply_id: item.id,
                    parent_id: comment_id.to_string(),
                    author: item.snippet.author_display_name,
                    author_channel_id: item.snippet.author_channel_id.value,
                    text: item.snippet.text_display,
                    like_count: item.snippet.like_count,
                    published_at: item.snippet.published_at,
                    ai_generated: false,
                    ai_model: None,
                    metadata: HashMap::new(),
                })
                .collect();

            all_replies.extend(replies);

            // Check if there are more pages
            if let Some(token) = response_data.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        Ok(all_replies)
    }

    /// Post a reply to a comment
    pub async fn post_reply(&self, user_id: &str, comment_id: &str, text: &str) -> Result<Reply> {
        info!("Posting reply to comment: {}", comment_id);

        // Get a valid access token
        let access_token = self.auth_service.get_valid_access_token(user_id).await?;

        // Create the request body
        let request_body = serde_json::json!({
            "snippet": {
                "parentId": comment_id,
                "textOriginal": text
            }
        });

        // Send the request
        let response = self.client
            .post("https://www.googleapis.com/youtube/v3/comments?part=snippet")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("YouTube API error: {}", error_text);
            anyhow::bail!("Failed to post reply: {}", error_text);
        }

        let response_data: YouTubeCommentItem = response.json().await?;

        // Create Reply object
        let reply = Reply {
            reply_id: response_data.id,
            parent_id: comment_id.to_string(),
            author: response_data.snippet.author_display_name,
            author_channel_id: response_data.snippet.author_channel_id.value,
            text: response_data.snippet.text_display,
            like_count: response_data.snippet.like_count,
            published_at: response_data.snippet.published_at,
            ai_generated: false, // This will be set by the caller if needed
            ai_model: None,
            metadata: HashMap::new(),
        };

        // Mark the comment as replied to
        self.db.mark_comment_replied(comment_id, true).await?;

        // Record the interaction
        let interaction = InteractionRecord {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            video_id: "".to_string(), // We don't have the video ID here
            comment_id: comment_id.to_string(),
            reply_id: Some(reply.reply_id.clone()),
            interaction_type: InteractionType::ReplyPosted,
            timestamp: Utc::now(),
            data: HashMap::new(),
        };

        self.db.record_interaction(&interaction).await?;

        Ok(reply)
    }

    /// Get videos for a channel
    pub async fn get_channel_videos(&self, user_id: &str) -> Result<Vec<YouTubeVideo>> {
        info!("Fetching videos for channel: {}", user_id);

        // Get a valid access token
        let access_token = self.auth_service.get_valid_access_token(user_id).await?;

        // First, get the channel ID for the authenticated user
        let url = "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true";

        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("YouTube API error: {}", error_text);
            anyhow::bail!("Failed to get channel ID: {}", error_text);
        }

        let channel_response: YouTubeChannelResponse = response.json().await?;

        if channel_response.items.is_empty() {
            anyhow::bail!("No channel found for the authenticated user");
        }

        let channel_id = &channel_response.items[0].id;

        // Now get the videos for this channel
        let mut all_videos = Vec::new();
        let mut page_token = None;

        loop {
            let url = format!(
                "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId={}&maxResults=50&order=date&type=video{}",
                channel_id,
                page_token.map_or(String::new(), |token| format!("&pageToken={}", token))
            );

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                error!("YouTube API error: {}", error_text);
                anyhow::bail!("Failed to get videos: {}", error_text);
            }

            let video_response: YouTubeVideoSearchResponse = response.json().await?;

            for item in video_response.items {
                all_videos.push(YouTubeVideo {
                    id: item.id.video_id,
                    title: item.snippet.title,
                    description: item.snippet.description,
                    published_at: item.snippet.published_at,
                    thumbnail_url: item.snippet.thumbnails.default.url,
                });
            }

            // Check if there are more pages
            if let Some(token) = video_response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }

            // Don't hit rate limits
            time::sleep(Duration::from_millis(100)).await;
        }

        Ok(all_videos)
    }

    /// Start monitoring for new comments
    pub async fn start_comment_monitor(&self, user_id: &str, polling_interval: u64) -> Result<()> {
        info!("Starting comment monitor for user: {}", user_id);

        // Get the user's videos
        let videos = self.get_channel_videos(user_id).await?;

        if videos.is_empty() {
            warn!("No videos found for user: {}", user_id);
            return Ok(());
        }

        info!("Found {} videos to monitor", videos.len());

        // Initial fetch of all comments
        for video in &videos {
            info!("Initial fetch of comments for video: {}", video.id);
            match self.fetch_comments(user_id, &video.id).await {
                Ok(comments) => {
                    info!("Fetched {} comments for video: {}", comments.len(), video.id);
                }
                Err(e) => {
                    error!("Error fetching comments for video {}: {}", video.id, e);
                }
            }

            // Don't hit rate limits
            time::sleep(Duration::from_millis(500)).await;
        }

        // TODO: Implement continuous monitoring in a separate task
        // This would typically be done with tokio::spawn and a loop with delay

        Ok(())
    }
}

// YouTube API response models

#[derive(Debug, Deserialize)]
struct YouTubeCommentThreadResponse {
    items: Vec<YouTubeCommentThread>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeCommentThread {
    id: String,
    snippet: YouTubeCommentThreadSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeCommentThreadSnippet {
    total_reply_count: i32,
    top_level_comment: YouTubeComment,
}

#[derive(Debug, Deserialize)]
struct YouTubeCommentResponse {
    items: Vec<YouTubeCommentItem>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeCommentItem {
    id: String,
    snippet: YouTubeCommentSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeComment {
    snippet: YouTubeCommentSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeCommentSnippet {
    author_display_name: String,
    author_channel_id: YouTubeChannelId,
    text_display: String,
    like_count: i32,
    published_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct YouTubeChannelId {
    value: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeChannelResponse {
    items: Vec<YouTubeChannelItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeChannelItem {
    id: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoSearchResponse {
    items: Vec<YouTubeVideoSearchItem>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoSearchItem {
    id: YouTubeVideoId,
    snippet: YouTubeVideoSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoId {
    video_id: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoSnippet {
    title: String,
    description: String,
    published_at: DateTime<Utc>,
    thumbnails: YouTubeThumbnails,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumbnails {
    default: YouTubeThumbnail,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumbnail {
    url: String,
}

/// YouTube video information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeVideo {
    /// Video ID
    pub id: String,

    /// Video title
    pub title: String,

    /// Video description
    pub description: String,

    /// When the video was published
    pub published_at: DateTime<Utc>,

    /// URL to the video thumbnail
    pub thumbnail_url: String,
}
