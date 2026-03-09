//! `OAuthService` — orchestrates OAuth providers and syncs user records.
//!
//! Mirrors MedusaJS's strategy pattern: a registry of named `OAuthProvider`
//! implementations that can be looked up by provider name at runtime.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::oauth::{OAuthProfile, OAuthProvider};
use crate::api::auth::AuthIdentity;

/// Central service for OAuth2 social login.
///
/// Register concrete providers (Google, Facebook, GitHub, …) at startup and
/// then call `handle_callback()` from the HTTP layer.
pub struct OAuthService {
    providers: HashMap<String, Arc<dyn OAuthProvider>>,
    db: PgPool,
}

impl OAuthService {
    pub fn new(db: PgPool) -> Self {
        Self {
            providers: HashMap::new(),
            db,
        }
    }

    /// Register an OAuth provider under its `name()`.
    pub fn register(&mut self, provider: Arc<dyn OAuthProvider>) {
        let name = provider.name().to_string();
        tracing::info!(provider = %name, "OAuth provider registered");
        self.providers.insert(name, provider);
    }

    /// Returns the authorization redirect URL for `provider_name`.
    pub fn authorization_url(
        &self,
        provider_name: &str,
        state: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<String> {
        let p = self
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("OAuth provider not found: {}", provider_name))?;
        p.authorization_url(state, redirect_uri)
    }

    /// Full callback flow: exchange `code` → fetch profile → upsert user.
    ///
    /// Returns the `AuthIdentity` (id + email) that can then be used to mint a
    /// JWT, exactly like the email/password flow does.
    pub async fn handle_callback(
        &self,
        provider_name: &str,
        code: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AuthIdentity> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("OAuth provider not found: {}", provider_name))?;

        let tokens = provider.exchange_code(code, redirect_uri).await?;
        let profile = provider.fetch_profile(&tokens).await?;

        tracing::info!(
            provider = %provider_name,
            provider_user_id = %profile.provider_user_id,
            email = ?profile.email,
            "OAuth profile fetched"
        );

        self.find_or_create_user(profile).await
    }

    /// Looks up or creates a user record for the given OAuth profile.
    ///
    /// The `oauth_users` join table (see migration
    /// `20260309000002_oauth_users.sql`) links a provider + provider-side ID to
    /// a local `users` row, mirroring the MedusaJS strategy pattern.
    async fn find_or_create_user(&self, profile: OAuthProfile) -> anyhow::Result<AuthIdentity> {
        // 1. Check for an existing oauth_users link.
        let row = sqlx::query(
            r#"
            SELECT u.id, u.email
            FROM users u
            INNER JOIN oauth_users ou ON ou.user_id = u.id
            WHERE ou.provider = $1 AND ou.provider_user_id = $2
            "#,
        )
        .bind(&profile.provider)
        .bind(&profile.provider_user_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let id: Uuid = r.get("id");
            let email: String = r.get("email");
            return Ok(AuthIdentity { id, email });
        }

        // 2. Require an email to create a new account.
        let email = profile
            .email
            .ok_or_else(|| anyhow::anyhow!("OAuth provider did not return an email address"))?;

        let mut tx = self.db.begin().await?;

        // 3. Upsert the users row by email (another provider may already have
        //    created it).  Use DO NOTHING on conflict so we do not accidentally
        //    overwrite any user-modified fields on subsequent OAuth logins.
        use sqlx::Row;
        let user_row = sqlx::query(
            r#"
            INSERT INTO users (email, first_name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (email) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&email)
        .bind(&profile.name)
        .fetch_optional(&mut *tx)
        .await?;

        // If the INSERT was a no-op (user already existed), look up the id.
        let user_id: Uuid = if let Some(row) = user_row {
            row.get("id")
        } else {
            sqlx::query("SELECT id FROM users WHERE email = $1")
                .bind(&email)
                .fetch_one(&mut *tx)
                .await?
                .get("id")
        };

        // 4. Insert the oauth_users link.
        sqlx::query(
            r#"
            INSERT INTO oauth_users (user_id, provider, provider_user_id, profile_data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (provider, provider_user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(&profile.provider)
        .bind(&profile.provider_user_id)
        .bind(&profile.raw)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(
            user_id = %user_id,
            email = %email,
            provider = %profile.provider,
            "User linked via OAuth"
        );

        Ok(AuthIdentity { id: user_id, email })
    }
}
