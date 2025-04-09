use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::Database;
use crate::models::auth::{AuthToken, Session, User, UserPreferences, ReplyTone};

/// YouTube OAuth2 configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Client ID from Google Cloud Console
    pub client_id: String,
    
    /// Client secret from Google Cloud Console
    pub client_secret: String,
    
    /// Redirect URI for OAuth flow
    pub redirect_uri: String,
    
    /// OAuth scopes required
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Create a new OAuth configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            client_id: env::var("YOUTUBE_OAUTH_CLIENT_ID")
                .context("YOUTUBE_OAUTH_CLIENT_ID environment variable not set")?,
            client_secret: env::var("YOUTUBE_OAUTH_CLIENT_SECRET")
                .context("YOUTUBE_OAUTH_CLIENT_SECRET environment variable not set")?,
            redirect_uri: env::var("YOUTUBE_OAUTH_REDIRECT_URI")
                .context("YOUTUBE_OAUTH_REDIRECT_URI environment variable not set")?,
            scopes: vec![
                "https://www.googleapis.com/auth/youtube.readonly".to_string(),
                "https://www.googleapis.com/auth/youtube.force-ssl".to_string(),
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
                "https://www.googleapis.com/auth/userinfo.profile".to_string(),
            ],
        })
    }
    
    /// Generate the authorization URL
    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "https://accounts.google.com/o/oauth2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}",
            self.client_id, self.redirect_uri, scopes, state
        )
    }
}

/// OAuth token response from Google
#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    token_type: String,
    scope: String,
}

/// User info response from Google
#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    id: String,
    email: Option<String>,
    name: String,
    picture: Option<String>,
}

/// Authentication service
pub struct AuthService {
    db: Database,
    client: Client,
    oauth_config: OAuthConfig,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(db: Database) -> Result<Self> {
        let oauth_config = OAuthConfig::from_env()?;
        let client = Client::new();
        
        Ok(Self {
            db,
            client,
            oauth_config,
        })
    }
    
    /// Generate an authorization URL for YouTube OAuth
    pub fn get_authorization_url(&self) -> String {
        let state = Uuid::new_v4().to_string();
        self.oauth_config.authorization_url(&state)
    }
    
    /// Exchange an authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<AuthToken> {
        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", &self.oauth_config.client_id),
                ("client_secret", &self.oauth_config.client_secret),
                ("code", &code.to_string()),
                ("grant_type", &"authorization_code".to_string()),
                ("redirect_uri", &self.oauth_config.redirect_uri),
            ])
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OAuth token error: {}", error_text);
            anyhow::bail!("Failed to exchange code for tokens: {}", error_text);
        }
        
        let token_response: OAuthTokenResponse = response.json().await?;
        
        // Convert to our AuthToken model
        let expires_at = Utc::now() + Duration::seconds(token_response.expires_in);
        let scopes = token_response.scope.split(' ').map(String::from).collect();
        
        Ok(AuthToken {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at,
            token_type: token_response.token_type,
            scopes,
        })
    }
    
    /// Refresh an access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken> {
        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", &self.oauth_config.client_id),
                ("client_secret", &self.oauth_config.client_secret),
                ("refresh_token", &refresh_token.to_string()),
                ("grant_type", &"refresh_token".to_string()),
            ])
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OAuth token refresh error: {}", error_text);
            anyhow::bail!("Failed to refresh token: {}", error_text);
        }
        
        let token_response: OAuthTokenResponse = response.json().await?;
        
        // Convert to our AuthToken model
        let expires_at = Utc::now() + Duration::seconds(token_response.expires_in);
        let scopes = token_response.scope.split(' ').map(String::from).collect();
        
        Ok(AuthToken {
            access_token: token_response.access_token,
            refresh_token: refresh_token.to_string(), // Keep the original refresh token
            expires_at,
            token_type: token_response.token_type,
            scopes,
        })
    }
    
    /// Get user information from Google
    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfoResponse> {
        let response = self.client
            .get("https://www.googleapis.com/oauth2/v1/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("User info error: {}", error_text);
            anyhow::bail!("Failed to get user info: {}", error_text);
        }
        
        let user_info: UserInfoResponse = response.json().await?;
        Ok(user_info)
    }
    
    /// Create or update a user from OAuth flow
    pub async fn create_or_update_user(&self, token: &AuthToken) -> Result<User> {
        // Get user info from Google
        let user_info = self.get_user_info(&token.access_token).await?;
        
        // Check if user exists
        let existing_user = self.db.get_user(&user_info.id).await?;
        
        let user = match existing_user {
            Some(mut user) => {
                // Update existing user
                user.name = user_info.name;
                user.email = user_info.email;
                user.profile_picture_url = user_info.picture;
                user.updated_at = Utc::now();
                user
            },
            None => {
                // Create new user
                let now = Utc::now();
                User {
                    id: user_info.id,
                    name: user_info.name,
                    email: user_info.email,
                    profile_picture_url: user_info.picture,
                    created_at: now,
                    updated_at: now,
                    preferences: UserPreferences {
                        enable_ai_replies: true,
                        ai_model: "gpt-3.5-turbo".to_string(),
                        reply_tone: ReplyTone::Friendly,
                        enable_notifications: true,
                        polling_interval: 60,
                        additional: Default::default(),
                    },
                    metadata: Default::default(),
                }
            }
        };
        
        // Save user to database
        self.db.save_user(&user).await?;
        
        // Save auth token
        self.db.save_auth_token(&user.id, token).await?;
        
        Ok(user)
    }
    
    /// Create a new session for a user
    pub async fn create_session(&self, user_id: &str, ip_address: &str, user_agent: &str) -> Result<Session> {
        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + Duration::days(7), // Session expires in 7 days
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            is_active: true,
        };
        
        self.db.create_session(&session).await?;
        
        Ok(session)
    }
    
    /// Validate a session
    pub async fn validate_session(&self, session_id: &str) -> Result<Option<(Session, User)>> {
        let session = match self.db.get_session(session_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        
        // Check if session is active and not expired
        if !session.is_active || session.expires_at < Utc::now() {
            return Ok(None);
        }
        
        // Get user
        let user = match self.db.get_user(&session.user_id).await? {
            Some(u) => u,
            None => return Ok(None),
        };
        
        Ok(Some((session, user)))
    }
    
    /// End a session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        self.db.end_session(session_id).await
    }
    
    /// Get a valid access token for a user
    pub async fn get_valid_access_token(&self, user_id: &str) -> Result<String> {
        let token = match self.db.get_auth_token(user_id).await? {
            Some(t) => t,
            None => anyhow::bail!("No auth token found for user {}", user_id),
        };
        
        // Check if token is expired or about to expire (within 5 minutes)
        if token.expires_at <= Utc::now() + Duration::minutes(5) {
            // Refresh the token
            let new_token = self.refresh_token(&token.refresh_token).await?;
            self.db.save_auth_token(user_id, &new_token).await?;
            Ok(new_token.access_token)
        } else {
            Ok(token.access_token)
        }
    }
}
