//! Authentication routes shared by store and admin.
//!
//! Implements the generic `/auth/:actor_type/:auth_provider` endpoints plus
//! session management and token refresh.  Logic is delegated to an
//! `AuthService` trait; the default `EmailPasswordService` provides basic
//! email/password authentication.

use axum::{extract::{Path, State}, Json, middleware::Next, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::{error::AppError, state::AppState};

mod service;
pub use service::{AuthService, AuthData, AuthIdentity, AuthResult, ActorType, EmailPasswordService};

// --- payloads ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AuthPayload(serde_json::Value);

// --- handlers ----------------------------------------------------------------

async fn provider(
    State(state): State<AppState>,
    Path((actor, provider)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = state.auth_service.clone();
    let actor_type = if actor == "customer" { ActorType::Customer } else { ActorType::User };

    let data = AuthData {
        url: "".into(),
        headers: Default::default(),
        query: Default::default(),
        body: payload,
        protocol: "".into(),
    };

    let result = service.authenticate(actor_type, &provider, data).await?;
    if let Some(location) = result.location {
        return Ok(Json(serde_json::json!({ "location": location })));
    }
    if result.success {
        if let Some(auth_id) = result.auth_identity {
            let token = if let ActorType::Customer = actor_type {
                crate::auth::jwt::encode_store_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            } else {
                crate::auth::jwt::encode_admin_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            };
            return Ok(Json(serde_json::json!({ "token": token })));
        }
    }
    Err(AppError::Unauthorized)
}

async fn callback(
    State(state): State<AppState>,
    Path((actor, provider)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = state.auth_service.clone();
    let actor_type = if actor == "customer" { ActorType::Customer } else { ActorType::User };

    let data = AuthData {
        url: "".into(),
        headers: Default::default(),
        query: Default::default(),
        body: payload,
        protocol: "".into(),
    };

    let result = service.validate_callback(actor_type, &provider, data).await?;
    if result.success {
        if let Some(auth_id) = result.auth_identity {
            let token = if let ActorType::Customer = actor_type {
                crate::auth::jwt::encode_store_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            } else {
                crate::auth::jwt::encode_admin_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            };
            return Ok(Json(serde_json::json!({ "token": token })));
        }
    }
    Err(AppError::Unauthorized)
}

async fn register(
    State(state): State<AppState>,
    Path((actor, provider)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = state.auth_service.clone();
    let actor_type = if actor == "customer" { ActorType::Customer } else { ActorType::User };

    let data = AuthData {
        url: "".into(),
        headers: Default::default(),
        query: Default::default(),
        body: payload,
        protocol: "".into(),
    };

    let result = service.register(actor_type, &provider, data).await?;
    if result.success {
        if let Some(auth_id) = result.auth_identity {
            let token = if let ActorType::Customer = actor_type {
                crate::auth::jwt::encode_store_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            } else {
                crate::auth::jwt::encode_admin_token(&auth_id.id, &auth_id.email, &state.jwt_secret, 24)?
            };
            return Ok(Json(serde_json::json!({ "token": token })));
        }
    }
    Err(AppError::Unauthorized)
}

async fn reset_password(
    State(state): State<AppState>,
    Path((actor, provider)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let service = state.auth_service.clone();
    let actor_type = if actor == "customer" { ActorType::Customer } else { ActorType::User };
    let data = AuthData { url: "".into(), headers: Default::default(), query: Default::default(), body: payload, protocol: "".into() };
    let result = service.reset_password(actor_type, &provider, data).await?;
    if result.success {
        return Ok(StatusCode::CREATED);
    }
    Err(AppError::Unauthorized)
}

async fn update(
    State(state): State<AppState>,
    Path((actor, provider)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = state.auth_service.clone();
    let actor_type = if actor == "customer" { ActorType::Customer } else { ActorType::User };
    let data = AuthData { url: "".into(), headers: Default::default(), query: Default::default(), body: payload, protocol: "".into() };
    let result = service.update(actor_type, &provider, data).await?;
    if result.success {
        return Ok(Json(serde_json::json!({ "success": true })));
    }
    Err(AppError::Unauthorized)
}

async fn session_post(
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    use crate::auth::jwt::{AuthAdmin, AuthCustomer};
    if let Some(AuthAdmin(claims)) = req.extensions().get::<AuthAdmin>() {
        let user = serde_json::json!({
            "actor_id": claims.sub,
            "email": claims.email,
            "role": "admin",
        });
        return (StatusCode::OK, Json(serde_json::json!({ "user": user }
        )));
    }
    if let Some(AuthCustomer(claims)) = req.extensions().get::<AuthCustomer>() {
        let user = serde_json::json!({
            "actor_id": claims.sub,
            "email": claims.email,
        });
        return (StatusCode::OK, Json(serde_json::json!({ "user": user })));
    }
    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "no auth" })))
}

async fn session_delete() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "session": "cleared" })))
}

async fn token_refresh(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::auth::jwt::{AuthAdmin, AuthCustomer};
    // check which claims were inserted by middleware
    if let Some(AuthAdmin(claims)) = req.extensions().get::<AuthAdmin>() {
        let uid: uuid::Uuid = claims.sub.parse().unwrap_or_default();
        let email = claims.email.clone();
        let token = crate::auth::jwt::encode_admin_token(&uid, &email, &state.jwt_secret, 24)?;
        return Ok(Json(serde_json::json!({ "token": token })));
    }
    if let Some(AuthCustomer(claims)) = req.extensions().get::<AuthCustomer>() {
        let uid: uuid::Uuid = claims.sub.parse().unwrap_or_default();
        let email = claims.email.clone();
        let token = crate::auth::jwt::encode_store_token(&uid, &email, &state.jwt_secret, 24)?;
        return Ok(Json(serde_json::json!({ "token": token })));
    }
    Err(AppError::Unauthorized)
}

/// Builds router for global auth endpoints.
pub fn auth_router(state: AppState) -> axum::Router<AppState> {
    use crate::routes_manifest::*;
    use axum::routing::{get, post, delete};

    let public = axum::Router::new()
        .route(AUTH_ACTOR_PROVIDER, get(provider).post(provider))
        .route(AUTH_ACTOR_PROVIDER_CALLBACK, get(callback).post(callback))
        .route(AUTH_ACTOR_PROVIDER_REGISTER, post(register))
        .route(AUTH_ACTOR_PROVIDER_RESET_PASSWORD, post(reset_password))
        .route(AUTH_ACTOR_PROVIDER_UPDATE, post(update));

    // session endpoints need authentication, so apply the general auth middleware
    let session = axum::Router::new()
        .route(AUTH_SESSION, post(session_post).delete(session_delete))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), crate::auth::jwt::general_auth_middleware));

    let refresh = axum::Router::new()
        .route(AUTH_TOKEN_REFRESH, post(token_refresh))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), crate::auth::jwt::general_auth_middleware));

    public.merge(session).merge(refresh)
}
