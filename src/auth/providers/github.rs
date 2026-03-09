//! GitHub OAuth2 provider.
//!
//! Enabled with Cargo feature `auth-github`.

#[cfg(feature = "auth-github")]
use crate::auth::oauth::{OAuthProfile, OAuthProvider, OAuthTokens};
#[cfg(feature = "auth-github")]
use async_trait::async_trait;
#[cfg(feature = "auth-github")]
use serde::Deserialize;

#[cfg(feature = "auth-github")]
const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
#[cfg(feature = "auth-github")]
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
#[cfg(feature = "auth-github")]
const USERINFO_URL: &str = "https://api.github.com/user";
#[cfg(feature = "auth-github")]
const EMAILS_URL: &str = "https://api.github.com/user/emails";

/// GitHub OAuth2 provider.  Requires the `auth-github` feature.
#[cfg(feature = "auth-github")]
pub struct GitHubOAuthProvider {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

#[cfg(feature = "auth-github")]
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[cfg(feature = "auth-github")]
#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[cfg(feature = "auth-github")]
#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    token_type: String,
    scope: Option<String>,
}

#[cfg(feature = "auth-github")]
impl GitHubOAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http: reqwest::Client::builder()
                .user_agent(concat!("medusa-rust/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[cfg(feature = "auth-github")]
#[async_trait]
impl OAuthProvider for GitHubOAuthProvider {
    fn name(&self) -> &str {
        "github"
    }

    fn authorization_url(&self, state: &str, redirect_uri: &str) -> anyhow::Result<String> {
        let url = format!(
            "{}?client_id={}&redirect_uri={}&scope=user:email&state={}",
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
        let resp: GitHubTokenResponse = self
            .http
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(OAuthTokens {
            access_token: resp.access_token,
            refresh_token: None,
            expires_in: None,
            token_type: resp.token_type,
        })
    }

    async fn fetch_profile(&self, tokens: &OAuthTokens) -> anyhow::Result<OAuthProfile> {
        let mut info: GitHubUserInfo = self
            .http
            .get(USERINFO_URL)
            .bearer_auth(&tokens.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        // If the public profile has no email, fall back to the /user/emails endpoint.
        if info.email.is_none() {
            if let Ok(emails) = self
                .http
                .get(EMAILS_URL)
                .bearer_auth(&tokens.access_token)
                .send()
                .await
                .and_then(|r| r.error_for_status())
            {
                if let Ok(list) = emails.json::<Vec<GitHubEmail>>().await {
                    info.email = list
                        .into_iter()
                        .find(|e| e.primary && e.verified)
                        .map(|e| e.email);
                }
            }
        }

        let raw = serde_json::json!({
            "id": info.id,
            "login": info.login,
            "name": info.name,
            "email": info.email,
            "avatar_url": info.avatar_url,
        });

        Ok(OAuthProfile {
            provider: "github".into(),
            provider_user_id: info.id.to_string(),
            email: info.email,
            name: info.name.or(Some(info.login)),
            avatar_url: info.avatar_url,
            raw,
        })
    }
}
