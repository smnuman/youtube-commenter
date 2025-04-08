use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

use crate::models::{Comment, Reply};

/// Fetch comments for a YouTube video
pub async fn fetch_comments(video_id: &str) -> Result<Vec<Comment>> {
    info!("Fetching comments for video: {}", video_id);
    
    let api_key = env::var("YOUTUBE_API_KEY")
        .context("YOUTUBE_API_KEY environment variable not set")?;
    
    let client = Client::new();
    
    // Fetch comment threads
    let comment_threads = fetch_comment_threads(&client, video_id, &api_key).await?;
    
    // Convert to our Comment model
    let mut comments = Vec::new();
    
    for thread in comment_threads {
        let comment_id = thread.id;
        let snippet = thread.snippet.top_level_comment.snippet;
        
        // Fetch replies if there are any
        let replies = if thread.snippet.total_reply_count > 0 {
            fetch_replies(&client, &comment_id, &api_key).await?
        } else {
            Vec::new()
        };
        
        comments.push(Comment {
            video_id: video_id.to_string(),
            comment_id,
            author: snippet.author_display_name,
            text: snippet.text_display,
            like_count: snippet.like_count,
            published_at: snippet.published_at,
            replies,
        });
    }
    
    info!("Fetched {} comments with replies", comments.len());
    
    Ok(comments)
}

/// Fetch comment threads from YouTube API
async fn fetch_comment_threads(
    client: &Client,
    video_id: &str,
    api_key: &str,
) -> Result<Vec<YouTubeCommentThread>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/commentThreads?part=snippet&videoId={}&key={}",
        video_id, api_key
    );
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("YouTube API error: {}", error_text);
        anyhow::bail!("YouTube API error: {}", error_text);
    }
    
    let response_data: YouTubeCommentThreadResponse = response.json().await?;
    
    Ok(response_data.items)
}

/// Fetch replies to a comment from YouTube API
async fn fetch_replies(
    client: &Client,
    comment_id: &str,
    api_key: &str,
) -> Result<Vec<Reply>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/comments?part=snippet&parentId={}&key={}",
        comment_id, api_key
    );
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("YouTube API error: {}", error_text);
        anyhow::bail!("YouTube API error: {}", error_text);
    }
    
    let response_data: YouTubeCommentResponse = response.json().await?;
    
    let replies = response_data
        .items
        .into_iter()
        .map(|item| Reply {
            reply_id: item.id,
            author: item.snippet.author_display_name,
            text: item.snippet.text_display,
            like_count: item.snippet.like_count,
            published_at: item.snippet.published_at,
        })
        .collect();
    
    Ok(replies)
}

// YouTube API response models

#[derive(Debug, Deserialize)]
struct YouTubeCommentThreadResponse {
    items: Vec<YouTubeCommentThread>,
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
    text_display: String,
    like_count: i32,
    published_at: DateTime<Utc>,
}
