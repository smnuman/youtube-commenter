/// Extract YouTube video ID from a URL
pub fn extract_video_id(input: &str) -> Option<String> {
    // Handle direct video IDs (11 characters)
    if input.len() == 11 && !input.contains('/') && !input.contains('?') {
        return Some(input.to_string());
    }
    
    // Try to parse as URL
    if let Ok(url) = url::Url::parse(input) {
        // Handle youtu.be format
        if url.host_str() == Some("youtu.be") {
            return url.path_segments()
                .and_then(|segments| segments.last())
                .map(|s| s.to_string());
        }
        
        // Handle youtube.com format
        if url.host_str() == Some("youtube.com") || url.host_str() == Some("www.youtube.com") {
            return url.query_pairs()
                .find(|(key, _)| key == "v")
                .map(|(_, value)| value.to_string());
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_video_id() {
        // Direct ID
        assert_eq!(
            extract_video_id("dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // youtu.be format
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // youtube.com format
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // Invalid input
        assert_eq!(extract_video_id("not-a-video-id"), None);
    }
}
