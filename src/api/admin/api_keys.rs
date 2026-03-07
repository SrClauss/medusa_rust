//! Admin api_keys handlers — DB-backed CRUD
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn d20() -> i64 { 20 }

fn ak_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "token": r.get::<String, _>("token"),
        "title": r.get::<String, _>("title"),
        "type": r.get::<String, _>("type"),
        "last_used_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_used_at"),
        "created_by": r.get::<Option<Uuid>, _>("created_by"),
        "revoked_by": r.get::<Option<Uuid>, _>("revoked_by"),
        "revoked_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("revoked_at"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

fn generate_token() -> String {
    use rand::Rng;
    let rng = rand::thread_rng();
    let token: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    format!("sk_{}", token)
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, token, title, type, last_used_at, created_by, revoked_by, revoked_at, created_at, updated_at \
         FROM api_keys ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&*state.db)
        .await?;
    let api_keys: Vec<_> = rows.iter().map(ak_json).collect();
    Ok(Json(serde_json::json!({"api_keys": api_keys, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, token, title, type, last_used_at, created_by, revoked_by, revoked_at, created_at, updated_at \
         FROM api_keys WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("API key not found".into()))?;
    Ok(Json(serde_json::json!({"api_key": ak_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let key_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("secret");
    let token = generate_token();
    sqlx::query(
        "INSERT INTO api_keys (id, token, title, type, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(&token)
    .bind(title)
    .bind(key_type)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE api_keys SET title = COALESCE($2, title), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "api-key", "deleted": true})))
}

pub async fn revoke(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE api_keys SET revoked_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
