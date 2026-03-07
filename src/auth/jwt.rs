//! JWT encode / decode and Axum extractor middleware.
#![allow(unused_imports)]

use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// JWT secret wrapper — avoids passing raw strings through the call stack.
#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            expiry_hours: 24,
        }
    }
}

/// Claims embedded in admin JWTs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    /// Subject (admin user id).
    pub sub: String,
    /// Email address of the admin.
    pub email: String,
    /// Expiry unix timestamp.
    pub exp: usize,
    /// Issued-at unix timestamp.
    pub iat: usize,
}

/// Claims embedded in store JWTs (customer-facing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreClaims {
    /// Subject (customer id).
    pub sub: String,
    /// Customer email address.
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

/// Generic claims alias used in middleware.
pub type Claims = AdminClaims;

/// Encodes an admin JWT with the given user information.
pub fn encode_admin_token(
    user_id: &Uuid,
    email: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = AdminClaims {
        sub: user_id.to_string(),
        email: email.to_owned(),
        exp: (now + Duration::hours(expiry_hours)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::from)
}

/// Encodes a store JWT for a customer.
pub fn encode_store_token(
    customer_id: &Uuid,
    email: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = StoreClaims {
        sub: customer_id.to_string(),
        email: email.to_owned(),
        exp: (now + Duration::hours(expiry_hours)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::from)
}

/// Decodes and validates an admin JWT, returning the embedded claims.
pub fn decode_admin_token(token: &str, secret: &str) -> Result<AdminClaims, AppError> {
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(AppError::from)?;
    Ok(token_data.claims)
}

/// Decodes and validates a store JWT.
pub fn decode_store_token(token: &str, secret: &str) -> Result<StoreClaims, AppError> {
    let token_data = decode::<StoreClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(AppError::from)?;
    Ok(token_data.claims)
}

/// Extracts a Bearer token from an `Authorization` header.
pub fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
}

// ─── Axum Extractors ──────────────────────────────────────────────────────────

/// Request-extension that carries validated admin claims after the auth
/// middleware has run.
#[derive(Clone, Debug)]
pub struct AuthAdmin(pub AdminClaims);

/// Request-extension that carries validated store customer claims.
#[derive(Clone, Debug)]
pub struct AuthCustomer(pub StoreClaims);

// ─── Axum middleware ─────────────────────────────────────────────────────────

/// Tower middleware layer that validates admin JWTs on every request in the
/// `/admin` scope (except the login route).
pub async fn admin_auth_middleware(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer(req.headers()).ok_or(AppError::Unauthorized)?;
    let claims = decode_admin_token(token, &state.jwt_secret)?;
    req.extensions_mut().insert(AuthAdmin(claims));
    Ok(next.run(req).await)
}

/// Tower middleware layer that validates store customer JWTs.
pub async fn store_auth_middleware(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer(req.headers()).ok_or(AppError::Unauthorized)?;
    let claims = decode_store_token(token, &state.jwt_secret)?;
    req.extensions_mut().insert(AuthCustomer(claims));
    Ok(next.run(req).await)
}
