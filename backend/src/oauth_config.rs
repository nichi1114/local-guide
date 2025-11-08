use anyhow::{Context, Result};

const DEFAULT_GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const DEFAULT_GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Clone, Debug)]
pub struct OAuthProviderConfig {
    pub provider_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub redirect_uri: String,
}

impl OAuthProviderConfig {
    pub fn load_from_env() -> Result<Vec<Self>> {
        Ok(vec![Self::google_from_env()?])
    }

    fn google_from_env() -> Result<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").context(
            "GOOGLE_CLIENT_ID environment variable is required for Google OAuth integration",
        )?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").context(
            "GOOGLE_CLIENT_SECRET environment variable is required for Google OAuth integration",
        )?;
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").context(
            "GOOGLE_REDIRECT_URI environment variable is required for Google OAuth integration",
        )?;

        let auth_url = std::env::var("GOOGLE_AUTH_URL")
            .unwrap_or_else(|_| DEFAULT_GOOGLE_AUTH_URL.to_string());
        let token_url = std::env::var("GOOGLE_TOKEN_URL")
            .unwrap_or_else(|_| DEFAULT_GOOGLE_TOKEN_URL.to_string());
        let userinfo_url = std::env::var("GOOGLE_USERINFO_URL")
            .unwrap_or_else(|_| DEFAULT_GOOGLE_USERINFO_URL.to_string());

        let provider_id =
            std::env::var("GOOGLE_PROVIDER_NAME").unwrap_or_else(|_| "google".to_string());

        Ok(Self {
            provider_id,
            client_id,
            client_secret,
            auth_url,
            token_url,
            userinfo_url,
            redirect_uri,
        })
    }
}
