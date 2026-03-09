//! Google OAuth2 provider.
//!
//! Enabled with Cargo feature `auth-google`.

#[cfg(feature = "auth-google")]
use crate::auth::oauth::{OAuthProfile, OAuthProvider, OAuthTokens};
#[cfg(feature = "auth-google")]
use async_trait::async_trait;
#[cfg(feature = "auth-google")]
use serde::Deserialize;

#[cfg(feature = "auth-google")]
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
#[cfg(feature = "auth-google")]
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
#[cfg(feature = "auth-google")]
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

/// Google OAuth2 provider.  Requires the `auth-google` feature.
#[cfg(feature = "auth-google")]
pub struct GoogleOAuthProvider {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

#[cfg(feature = "auth-google")]
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

#[cfg(feature = "auth-google")]
#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
}

#[cfg(feature = "auth-google")]
impl GoogleOAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "auth-google")]
#[async_trait]
impl OAuthProvider for GoogleOAuthProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn authorization_url(&self, state: &str, redirect_uri: &str) -> anyhow::Result<String> {
        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=email+profile&state={}",
            AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state),
        );
        Ok(url)
    }

    async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<OAuthTokens> {
        let resp: GoogleTokenResponse = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("code", code),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(OAuthTokens {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_in: resp.expires_in,
            token_type: resp.token_type,
        })
    }

    async fn fetch_profile(&self, tokens: &OAuthTokens) -> anyhow::Result<OAuthProfile> {
        let info: GoogleUserInfo = self
            .http
            .get(USERINFO_URL)
            .bearer_auth(&tokens.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let raw = serde_json::json!({
            "id": info.id,
            "email": info.email,
            "name": info.name,
            "picture": info.picture,
        });

        Ok(OAuthProfile {
            provider: "google".into(),
            provider_user_id: info.id,
            email: info.email,
            name: info.name,
            avatar_url: info.picture,
            raw,
        })
    }
}
