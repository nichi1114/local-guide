use anyhow::{Context, Result};
use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier,
    RedirectUrl, TokenResponse, TokenUrl,
};
use reqwest::Url;
use serde::Deserialize;
use thiserror::Error;

use crate::oauth_config::OAuthProviderConfig;
use crate::repository::auth::{AuthRepository, IdentityProfile, UserRecord};

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
    client: BasicClient,
    userinfo_url: Url,
    http_client: reqwest::Client,
    provider_id: String,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub user: UserRecord,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("storage failure")]
    Storage(#[source] anyhow::Error),
    #[error("failed to exchange authorization code")]
    TokenExchange(#[source] anyhow::Error),
    #[error("failed to fetch user info")]
    UserInfo(#[source] anyhow::Error),
}

impl AuthService {
    pub fn new(repository: AuthRepository, config: OAuthProviderConfig) -> Result<Self> {
        let auth_url =
            AuthUrl::new(config.auth_url.clone()).context("invalid authorization url")?;
        let token_url = TokenUrl::new(config.token_url.clone()).context("invalid token url")?;
        let redirect_url =
            RedirectUrl::new(config.redirect_uri.clone()).context("invalid redirect url")?;

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        let userinfo_url = Url::parse(&config.userinfo_url).context("invalid userinfo url")?;
        let http_client = reqwest::Client::builder().build()?;

        Ok(Self {
            repository,
            client,
            userinfo_url,
            http_client,
            provider_id: config.provider_id,
        })
    }

    pub async fn complete_oauth_flow(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<AuthSession, AuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .set_pkce_verifier(PkceCodeVerifier::new(code_verifier.to_owned()))
            .request_async(async_http_client)
            .await
            .map_err(|error| AuthError::TokenExchange(error.into()))?;

        let access_token = token_response.access_token().secret().to_owned();
        let refresh_token = token_response
            .refresh_token()
            .map(|value| value.secret().to_owned());
        let expires_in = token_response
            .expires_in()
            .map(|duration| duration.as_secs());

        let profile = self
            .fetch_user_info(&access_token)
            .await
            .map_err(AuthError::UserInfo)?;

        let user = self
            .repository
            .upsert_user_with_identity(IdentityProfile {
                provider: &self.provider_id,
                provider_user_id: &profile.sub,
                email: profile.email.as_deref(),
                name: profile.name.as_deref(),
                avatar_url: profile.picture.as_deref(),
            })
            .await
            .map_err(AuthError::Storage)?;

        Ok(AuthSession {
            user,
            access_token,
            refresh_token,
            expires_in,
        })
    }

    async fn fetch_user_info(&self, access_token: &str) -> Result<ProviderUserInfo> {
        let response = self
            .http_client
            .get(self.userinfo_url.clone())
            .bearer_auth(access_token)
            .send()
            .await
            .context("failed to send user info request")?
            .error_for_status()
            .context("userinfo endpoint returned error")?;

        let profile = response
            .json::<ProviderUserInfo>()
            .await
            .context("failed to decode user info response")?;

        Ok(profile)
    }
}

#[derive(Debug, Deserialize)]
struct ProviderUserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_pool;
    use crate::repository::auth::AuthRepository;
    use crate::sql_init::run_initialization;
    use sqlx::PgPool;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/local_guide_test".into()
            });

        let pool = create_pool(&database_url)
            .await
            .expect("failed to connect to postgres");
        run_initialization(&pool)
            .await
            .expect("failed to apply initialization SQL");

        sqlx::query("TRUNCATE TABLE oauth_identities, users RESTART IDENTITY")
            .execute(&pool)
            .await
            .expect("failed to truncate tables");

        pool
    }

    fn build_test_config(mock_server: &MockServer) -> OAuthProviderConfig {
        OAuthProviderConfig {
            provider_id: "google".to_string(),
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            auth_url: format!("{}/auth", mock_server.uri()),
            token_url: format!("{}/token", mock_server.uri()),
            userinfo_url: format!("{}/userinfo", mock_server.uri()),
            redirect_uri: "https://example.com/callback".to_string(),
        }
    }

    #[tokio::test]
    async fn exchanges_code_and_links_user() {
        let pool = setup_pool().await;
        let repository = AuthRepository::new(pool.clone());
        let mock_server = MockServer::start().await;
        let config = build_test_config(&mock_server);
        let service = AuthService::new(repository.clone(), config).expect("service init");

        let access_token_response = serde_json::json!({
            "access_token": "mock-access-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "mock-refresh-token"
        });

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(access_token_response))
            .mount(&mock_server)
            .await;

        let user_info_response = serde_json::json!({
            "sub": "google-user-123",
            "email": "user@example.com",
            "name": "Test User",
            "picture": "https://example.com/avatar.png"
        });

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .and(header("authorization", "Bearer mock-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user_info_response))
            .mount(&mock_server)
            .await;

        let session = service
            .complete_oauth_flow("test-code", "test-verifier")
            .await
            .expect("exchange code");

        assert_eq!(session.access_token, "mock-access-token");
        assert_eq!(session.refresh_token.as_deref(), Some("mock-refresh-token"));
        assert_eq!(session.expires_in, Some(3600));
        assert_eq!(session.user.email.as_deref(), Some("user@example.com"));
        assert_eq!(session.user.name.as_deref(), Some("Test User"));
    }

    #[tokio::test]
    async fn reuses_identity_and_updates_profile() {
        let pool = setup_pool().await;
        let repository = AuthRepository::new(pool.clone());
        let mock_server = MockServer::start().await;
        let config = build_test_config(&mock_server);
        let service = AuthService::new(repository.clone(), config).expect("service init");

        let access_token_response = serde_json::json!({
            "access_token": "mock-access-token",
            "token_type": "Bearer"
        });

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(access_token_response.clone()))
            .mount(&mock_server)
            .await;

        let initial_profile = serde_json::json!({
            "sub": "google-user-123",
            "email": "user@example.com",
            "name": "Test User",
            "picture": "https://example.com/avatar.png"
        });

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .and(header("authorization", "Bearer mock-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(initial_profile))
            .mount(&mock_server)
            .await;

        service
            .complete_oauth_flow("code-1", "verifier-1")
            .await
            .expect("first exchange succeeds");

        mock_server.reset().await;

        let access_token_response_second = serde_json::json!({
            "access_token": "mock-access-token",
            "token_type": "Bearer"
        });

        let updated_profile = serde_json::json!({
            "sub": "google-user-123",
            "email": serde_json::Value::Null,
            "name": "Updated Name",
            "picture": serde_json::Value::Null
        });

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(access_token_response_second))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .and(header("authorization", "Bearer mock-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_profile))
            .mount(&mock_server)
            .await;

        let session = service
            .complete_oauth_flow("code-2", "verifier-2")
            .await
            .expect("second exchange succeeds");

        assert_eq!(session.user.name.as_deref(), Some("Updated Name"));
        assert_eq!(
            session.user.avatar_url.as_deref(),
            Some("https://example.com/avatar.png")
        );
    }
}
