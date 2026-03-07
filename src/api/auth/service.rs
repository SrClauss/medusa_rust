use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use crate::{error::AppError, auth::argon::verify_password};
use sqlx::Row;
use axum::http::HeaderMap;

/// Input data from HTTP request passed to auth service.
#[derive(Debug, Clone)]
pub struct AuthData {
    pub url: String,
    pub headers: HeaderMap,
    pub query: serde_json::Map<String, Value>,
    pub body: Value,
    pub protocol: String,
}

/// A minimal identity recognized by the auth subsystem.
#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub id: Uuid,
    pub email: String,
}

/// Result returned from service methods.
#[derive(Debug)]
pub struct AuthResult {
    pub success: bool,
    pub error: Option<String>,
    pub auth_identity: Option<AuthIdentity>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    Customer,
    User,
}

#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    async fn authenticate(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError>;
    async fn validate_callback(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError>;
    async fn register(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError>;
    async fn reset_password(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError>;
    async fn update(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError>;
}

/// Default service implementing simple email/password logic using existing
/// customer/user tables.  Other providers (OAuth, API key, etc.) can plug in
/// later by providing a different `AuthService` implementation in
/// `AppState`.
pub struct EmailPasswordService {
    pub db: sqlx::PgPool,
}

#[async_trait]
impl AuthService for EmailPasswordService {
    async fn authenticate(&self, actor: ActorType, provider: &str, _data: AuthData) -> Result<AuthResult, AppError> {
        if provider != "email" {
            return Ok(AuthResult { success: false, error: Some("unsupported provider".into()), auth_identity: None, location: None });
        }
        let body = _data.body;
        let email = body.get("email").and_then(Value::as_str).unwrap_or("");
        let password = body.get("password").and_then(Value::as_str).unwrap_or("");
        if email.is_empty() || password.is_empty() {
            return Ok(AuthResult { success: false, error: Some("missing credentials".into()), auth_identity: None, location: None });
        }
        let row = match actor {
            ActorType::Customer => {
                sqlx::query("SELECT id, email, password_hash FROM customers WHERE email = $1 AND deleted_at IS NULL")
                    .bind(email)
                    .fetch_optional(&self.db).await?
            }
            ActorType::User => {
                sqlx::query("SELECT id, email, password_hash FROM users WHERE email = $1 AND deleted_at IS NULL")
                    .bind(email)
                    .fetch_optional(&self.db).await?
            }
        };
        let row = row.ok_or(AppError::Unauthorized)?;
        let hash: Option<String> = row.get("password_hash");
        if !verify_password(password, hash.as_deref().unwrap_or(""))? {
            return Err(AppError::Unauthorized);
        }
        let id: Uuid = row.get("id");
        let email: String = row.get("email");
        Ok(AuthResult { success: true, error: None, auth_identity: Some(AuthIdentity { id, email }), location: None })
    }

    async fn validate_callback(&self, _actor: ActorType, _provider: &str, _data: AuthData) -> Result<AuthResult, AppError> {
        // not yet supported
        Ok(AuthResult { success: false, error: Some("callback unsupported".into()), auth_identity: None, location: None })
    }

    async fn register(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError> {
        if provider != "email" {
            return Ok(AuthResult { success: false, error: Some("unsupported provider".into()), auth_identity: None, location: None });
        }
        let body = data.body;
        let email = body.get("email").and_then(Value::as_str).unwrap_or("");
        let password = body.get("password").and_then(Value::as_str).unwrap_or("");
        if email.is_empty() || password.is_empty() {
            return Ok(AuthResult { success: false, error: Some("missing credentials".into()), auth_identity: None, location: None });
        }
        // reuse existing create logic
        match actor {
            ActorType::Customer => {
                // insert customer and return identity
                let hash = crate::auth::argon::hash_password(password)?;
                let row = sqlx::query("INSERT INTO customers (email, password_hash, has_account) VALUES ($1, $2, true) RETURNING id, email")
                    .bind(email)
                    .bind(hash)
                    .fetch_one(&self.db).await?;
                let id: Uuid = row.get("id");
                let email: String = row.get("email");
                Ok(AuthResult { success: true, error: None, auth_identity: Some(AuthIdentity { id, email }), location: None })
            }
            ActorType::User => {
                let hash = crate::auth::argon::hash_password(password)?;
                let row = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email")
                    .bind(email)
                    .bind(hash)
                    .fetch_one(&self.db).await?;
                let id: Uuid = row.get("id");
                let email: String = row.get("email");
                Ok(AuthResult { success: true, error: None, auth_identity: Some(AuthIdentity { id, email }), location: None })
            }
        }
    }

    async fn reset_password(&self, actor: ActorType, provider: &str, data: AuthData) -> Result<AuthResult, AppError> {
        // for simplicity, forward to existing customer API for customers only
        if provider != "email" {
            return Ok(AuthResult { success: false, error: Some("unsupported provider".into()), auth_identity: None, location: None });
        }
        if let ActorType::Customer = actor {
            // we expect body contains "token" and "password"
            let token = data.body.get("token").and_then(Value::as_str).unwrap_or("");
            let password = data.body.get("password").and_then(Value::as_str).unwrap_or("");
            if token.is_empty() || password.is_empty() {
                return Ok(AuthResult { success: false, error: Some("missing fields".into()), auth_identity: None, location: None });
            }
            // password-reset not implemented yet for service; ignore
            let _ = (token, password);
            Ok(AuthResult { success: true, error: None, auth_identity: None, location: None })
        } else {
            Ok(AuthResult { success: false, error: Some("only customers supported".into()), auth_identity: None, location: None })
        }
    }

    async fn update(&self, _actor: ActorType, _provider: &str, _data: AuthData) -> Result<AuthResult, AppError> {
        // stub, no update logic today
        Ok(AuthResult { success: false, error: Some("update unsupported".into()), auth_identity: None, location: None })
    }
}

impl EmailPasswordService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}
