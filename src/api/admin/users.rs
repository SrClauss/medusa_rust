//! Admin users handlers — full CRUD
use axum::{extract::{Extension, Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{auth::jwt::AuthAdmin, error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn d20() -> i64 { 20 }

fn user_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "email": r.get::<String, _>("email"),
        "first_name": r.get::<Option<String>, _>("first_name"),
        "last_name": r.get::<Option<String>, _>("last_name"),
        "role": r.get::<String, _>("role"),
        "api_token": r.get::<Option<String>, _>("api_token"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at \
         FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?;
    let users: Vec<_> = rows.iter().map(|r| user_json(r)).collect();
    Ok(Json(serde_json::json!({ "users": users, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthAdmin>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let r = sqlx::query(
        "SELECT id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at \
         FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(Json(serde_json::json!({ "user": user_json(&r) })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at \
         FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(Json(serde_json::json!({ "user": user_json(&r) })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let email = payload.get("email").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("member");
    let hash = payload.get("password").and_then(|v| v.as_str())
        .map(|p| crate::auth::argon::hash_password(p))
        .transpose()?;
    let r = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, password_hash, metadata, \
         created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW()) \
         RETURNING id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(email)
    .bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(role)
    .bind(hash)
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "user": user_json(&r) }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE users SET \
         first_name = COALESCE($2, first_name), \
         last_name = COALESCE($3, last_name), \
         role = COALESCE($4, role), \
         metadata = COALESCE($5, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("role").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(Json(serde_json::json!({ "user": user_json(&r) })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "user", "deleted": true })))
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("email required".into()))?;
    // Generate a reset token and store it (email sending not yet implemented).
    // We always return success to prevent email enumeration.
    let token = uuid::Uuid::new_v4().to_string();
    let _ = sqlx::query(
        "UPDATE users SET api_token = $2, updated_at = NOW() WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(email)
    .bind(&token)
    .execute(&*state.db)
    .await;
    Ok(Json(serde_json::json!({ "email": email })))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = payload.get("token").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("token required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("password required".into()))?;
    let hash = crate::auth::argon::hash_password(password)?;
    let updated = sqlx::query(
        "UPDATE users SET password_hash = $2, api_token = NULL, updated_at = NOW() \
         WHERE api_token = $1 AND deleted_at IS NULL",
    )
    .bind(token)
    .bind(hash)
    .execute(&*state.db)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::BadRequest("Invalid or expired token".into()));
    }
    Ok(Json(serde_json::json!({ "user": {} })))
}
