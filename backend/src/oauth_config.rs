use std::env::{self, VarError};
use thiserror::Error;

const DEFAULT_GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const DEFAULT_GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Clone, Debug)]
pub struct OAuthProviderConfig {
    pub provider_id: String,
    pub client_id: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub redirect_uri: String,
}

#[derive(Debug, Error)]
pub enum OAuthConfigError {
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("invalid unicode in environment variable {0}")]
    InvalidUnicode(&'static str),
    #[error("no OAuth providers configured")]
    NoProviders,
}

impl OAuthProviderConfig {
    pub fn load_from_env() -> Result<Vec<Self>, OAuthConfigError> {
        let providers = GOOGLE_ENV_VARIANTS
            .iter()
            .filter_map(|variant| Self::google_from_variant(variant).transpose())
            .collect::<Result<Vec<_>, _>>()?;

        if providers.is_empty() {
            return Err(OAuthConfigError::NoProviders);
        }

        Ok(providers)
    }

    fn google_from_variant(variant: &GoogleEnvVariant) -> Result<Option<Self>, OAuthConfigError> {
        let client_id = match env::var(variant.client_id_key) {
            Ok(value) => value,
            Err(VarError::NotPresent) => return Ok(None),
            Err(VarError::NotUnicode(_)) => {
                return Err(OAuthConfigError::InvalidUnicode(variant.client_id_key))
            }
        };

        let redirect_uri = read_required_env(variant.redirect_uri_key)?;

        let auth_url =
            env::var("GOOGLE_AUTH_URL").unwrap_or_else(|_| DEFAULT_GOOGLE_AUTH_URL.to_string());
        let token_url =
            env::var("GOOGLE_TOKEN_URL").unwrap_or_else(|_| DEFAULT_GOOGLE_TOKEN_URL.to_string());
        let userinfo_url = env::var("GOOGLE_USERINFO_URL")
            .unwrap_or_else(|_| DEFAULT_GOOGLE_USERINFO_URL.to_string());

        let provider_id = variant
            .provider_name_key
            .and_then(|key| env::var(key).ok())
            .unwrap_or_else(|| variant.default_provider_id.to_string());

        Ok(Some(Self {
            provider_id,
            client_id,
            auth_url,
            token_url,
            userinfo_url,
            redirect_uri,
        }))
    }
}

fn read_required_env(key: &'static str) -> Result<String, OAuthConfigError> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(VarError::NotPresent) => Err(OAuthConfigError::MissingEnv(key)),
        Err(VarError::NotUnicode(_)) => Err(OAuthConfigError::InvalidUnicode(key)),
    }
}

struct GoogleEnvVariant {
    client_id_key: &'static str,
    redirect_uri_key: &'static str,
    provider_name_key: Option<&'static str>,
    default_provider_id: &'static str,
}

impl GoogleEnvVariant {
    const fn new(
        client_id_key: &'static str,
        redirect_uri_key: &'static str,
        provider_name_key: Option<&'static str>,
        default_provider_id: &'static str,
    ) -> Self {
        Self {
            client_id_key,
            redirect_uri_key,
            provider_name_key,
            default_provider_id,
        }
    }
}

const GOOGLE_ENV_VARIANTS: &[GoogleEnvVariant] = &[
    GoogleEnvVariant::new(
        "GOOGLE_IOS_CLIENT_ID",
        "GOOGLE_IOS_REDIRECT_URI",
        Some("GOOGLE_IOS_PROVIDER_NAME"),
        "google-ios",
    ),
    GoogleEnvVariant::new(
        "GOOGLE_ANDROID_CLIENT_ID",
        "GOOGLE_ANDROID_REDIRECT_URI",
        Some("GOOGLE_ANDROID_PROVIDER_NAME"),
        "google-android",
    ),
];
