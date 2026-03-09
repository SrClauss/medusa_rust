//! OAuth2 provider trait and shared data types.
//!
//! Define the `OAuthProvider` trait that every social-login backend must
//! implement (Google, Facebook, GitHub, …).  Concrete implementations live in
//! `src/auth/providers/` and are compiled only when the corresponding Cargo
//! feature is enabled.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Profile data returned by an OAuth provider after a successful login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProfile {
    /// Provider name, e.g. `"google"`, `"facebook"`, `"github"`.
    pub provider: String,
    /// Unique user identifier within the provider's system.
    pub provider_user_id: String,
    /// User email, when the provider exposes it.
    pub email: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Avatar / profile-picture URL.
    pub avatar_url: Option<String>,
    /// Raw JSON payload from the provider's user-info endpoint.
    pub raw: serde_json::Value,
}

/// Token bundle returned by an OAuth provider after code exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Lifetime in seconds, when returned by the provider.
    pub expires_in: Option<u64>,
    pub token_type: String,
}

/// Trait that every OAuth2 provider backend must implement.
///
/// The flow mirrors MedusaJS:
/// 1. Redirect the user to `authorization_url()`.
/// 2. Receive the `code` callback and call `exchange_code()`.
/// 3. Use the returned tokens to `fetch_profile()`.
/// 4. Hand the profile to `OAuthService::find_or_create_user()`.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Short lower-case provider name, e.g. `"google"`.
    fn name(&self) -> &str;

    /// Builds the authorization URL the user must be redirected to.
    ///
    /// `state` is a CSRF token that you must verify in the callback.
    fn authorization_url(&self, state: &str, redirect_uri: &str) -> anyhow::Result<String>;

    /// Exchanges the `code` received in the callback for access / refresh tokens.
    async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<OAuthTokens>;

    /// Fetches the authenticated user's profile using the access token.
    async fn fetch_profile(&self, tokens: &OAuthTokens) -> anyhow::Result<OAuthProfile>;
}
