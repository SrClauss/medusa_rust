//! Facebook OAuth2 provider.
//!
//! Enabled with Cargo feature `auth-facebook`.

#[cfg(feature = "auth-facebook")]
use crate::auth::oauth::{OAuthProfile, OAuthProvider, OAuthTokens};
#[cfg(feature = "auth-facebook")]
use async_trait::async_trait;
#[cfg(feature = "auth-facebook")]
use serde::Deserialize;

#[cfg(feature = "auth-facebook")]
const AUTH_URL: &str = "https://www.facebook.com/v18.0/dialog/oauth";
#[cfg(feature = "auth-facebook")]
const TOKEN_URL: &str = "https://graph.facebook.com/v18.0/oauth/access_token";
#[cfg(feature = "auth-facebook")]
const USERINFO_URL: &str = "https://graph.facebook.com/me?fields=id,name,email,picture";

/// Facebook OAuth2 provider.  Requires the `auth-facebook` feature.
#[cfg(feature = "auth-facebook")]
pub struct FacebookOAuthProvider {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

#[cfg(feature = "auth-facebook")]
#[derive(Debug, Deserialize)]
struct FacebookPicture {
    data: FacebookPictureData,
}

#[cfg(feature = "auth-facebook")]
#[derive(Debug, Deserialize)]
struct FacebookPictureData {
    url: String,
}

#[cfg(feature = "auth-facebook")]
#[derive(Debug, Deserialize)]
struct FacebookUserInfo {
    id: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<FacebookPicture>,
}

#[cfg(feature = "auth-facebook")]
#[derive(Debug, Deserialize)]
struct FacebookTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
}

#[cfg(feature = "auth-facebook")]
impl FacebookOAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "auth-facebook")]
#[async_trait]
impl OAuthProvider for FacebookOAuthProvider {
    fn name(&self) -> &str {
        "facebook"
    }

    fn authorization_url(&self, state: &str, redirect_uri: &str) -> anyhow::Result<String> {
        let url = format!(
            "{}?client_id={}&redirect_uri={}&scope=email&state={}",
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
        let resp: FacebookTokenResponse = self
            .http
            .get(TOKEN_URL)
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("redirect_uri", redirect_uri),
                ("code", code),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(OAuthTokens {
            access_token: resp.access_token,
            refresh_token: None,
            expires_in: resp.expires_in,
            token_type: resp.token_type,
        })
    }

    async fn fetch_profile(&self, tokens: &OAuthTokens) -> anyhow::Result<OAuthProfile> {
        let info: FacebookUserInfo = self
            .http
            .get(USERINFO_URL)
            .bearer_auth(&tokens.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let avatar_url = info
            .picture
            .as_ref()
            .map(|p| p.data.url.clone());

        let raw = serde_json::json!({
            "id": info.id,
            "name": info.name,
            "email": info.email,
        });

        Ok(OAuthProfile {
            provider: "facebook".into(),
            provider_user_id: info.id,
            email: info.email,
            name: info.name,
            avatar_url,
            raw,
        })
    }
}
