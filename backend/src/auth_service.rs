use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, PkceCodeVerifier, RedirectUrl,
    TokenResponse, TokenUrl,
};
use reqwest::Url;
use serde::Deserialize;
use thiserror::Error;

use crate::oauth_config::OAuthProviderConfig;
use crate::repository::auth::{AuthRepository, AuthRepositoryError, IdentityProfile, UserRecord};

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
    client: BasicClient,
    userinfo_url: Url,
    http_client: reqwest::Client,
    provider_id: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("storage failure: {0}")]
    Storage(#[from] AuthRepositoryError),
    #[error("failed to exchange authorization code: {0}")]
    TokenExchange(String),
    #[error("failed to fetch user info: {0}")]
    UserInfo(String),
}

#[derive(Debug, Error)]
pub enum AuthServiceBuildError {
    #[error("invalid authorization url: {0}")]
    InvalidAuthUrl(String),
    #[error("invalid token url: {0}")]
    InvalidTokenUrl(String),
    #[error("invalid redirect url: {0}")]
    InvalidRedirectUrl(String),
    #[error("invalid userinfo url: {0}")]
    InvalidUserInfoUrl(String),
    #[error("failed to build HTTP client: {0}")]
    HttpClient(#[from] reqwest::Error),
}

impl AuthService {
    pub fn new(
        repository: AuthRepository,
        config: OAuthProviderConfig,
    ) -> Result<Self, AuthServiceBuildError> {
        let auth_url = AuthUrl::new(config.auth_url.clone())
            .map_err(|err| AuthServiceBuildError::InvalidAuthUrl(err.to_string()))?;
        let token_url = TokenUrl::new(config.token_url.clone())
            .map_err(|err| AuthServiceBuildError::InvalidTokenUrl(err.to_string()))?;
        let redirect_url = RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|err| AuthServiceBuildError::InvalidRedirectUrl(err.to_string()))?;

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            None,
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        let userinfo_url = Url::parse(&config.userinfo_url)
            .map_err(|err| AuthServiceBuildError::InvalidUserInfoUrl(err.to_string()))?;
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
        code_verifier: Option<&str>,
    ) -> Result<UserRecord, AuthError> {
        let mut request = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()));

        if let Some(verifier) = code_verifier {
            request = request.set_pkce_verifier(PkceCodeVerifier::new(verifier.to_owned()));
        }

        let token_response = request
            .request_async(async_http_client)
            .await
            .map_err(|error| AuthError::TokenExchange(error.to_string()))?;

        let access_token = token_response.access_token().secret().to_owned();
        let profile = self.fetch_user_info(&access_token).await?;

        let user = self
            .repository
            .upsert_user_with_identity(IdentityProfile {
                provider: &self.provider_id,
                provider_user_id: &profile.sub,
                email: profile.email.as_deref(),
                name: profile.name.as_deref(),
                avatar_url: profile.picture.as_deref(),
            })
            .await?;

        Ok(user)
    }

    async fn fetch_user_info(&self, access_token: &str) -> Result<ProviderUserInfo, AuthError> {
        let response = self
            .http_client
            .get(self.userinfo_url.clone())
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|err| AuthError::UserInfo(format!("failed to send user info request: {err}")))?
            .error_for_status()
            .map_err(|err| {
                AuthError::UserInfo(format!("userinfo endpoint returned error: {err}"))
            })?;

        let profile = response.json::<ProviderUserInfo>().await.map_err(|err| {
            AuthError::UserInfo(format!("failed to decode user info response: {err}"))
        })?;

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

        let user = service
            .complete_oauth_flow("test-code", Some("test-verifier"))
            .await
            .expect("exchange code");

        assert_eq!(user.email.as_deref(), Some("user@example.com"));
        assert_eq!(user.name.as_deref(), Some("Test User"));
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
            .complete_oauth_flow("code-1", Some("verifier-1"))
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

        let user = service
            .complete_oauth_flow("code-2", Some("verifier-2"))
            .await
            .expect("second exchange succeeds");

        assert_eq!(user.name.as_deref(), Some("Updated Name"));
        assert_eq!(
            user.avatar_url.as_deref(),
            Some("https://example.com/avatar.png")
        );
    }
}
