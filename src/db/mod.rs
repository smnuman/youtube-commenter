use anyhow::{Context, Result};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use tracing::info;

use crate::models::Comment;

pub type Database = Surreal<Db>;

/// Initialize the SurrealDB database
pub async fn init_db() -> Result<Database> {
    info!("Initializing SurrealDB");
    
    // Create an in-memory database
    let db = Surreal::new::<Mem>(()).await?;
    
    // Select a namespace and database
    db.use_ns("youtube_commenter").use_db("comments").await?;
    
    // Create schema
    db.query("DEFINE TABLE comments SCHEMAFULL")
        .await?;
    
    db.query(r#"
        DEFINE FIELD video_id ON TABLE comments TYPE string;
        DEFINE FIELD comment_id ON TABLE comments TYPE string;
        DEFINE FIELD author ON TABLE comments TYPE string;
        DEFINE FIELD text ON TABLE comments TYPE string;
        DEFINE FIELD like_count ON TABLE comments TYPE int;
        DEFINE FIELD published_at ON TABLE comments TYPE datetime;
        DEFINE FIELD replies ON TABLE comments TYPE array;
        DEFINE INDEX video_id_idx ON TABLE comments COLUMNS video_id;
    "#)
    .await?;
    
    info!("SurrealDB initialized successfully");
    
    Ok(db)
}

impl Database {
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
}
