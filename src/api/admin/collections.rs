//! Admin collection handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM product_collections WHERE deleted_at IS NULL")
        .fetch_one(&*state.db).await?;
    let collections: Vec<_> = rows.iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"handle":r.get::<String,_>("handle"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"deleted_at":null,"products":[]})).collect();
    Ok(Json(serde_json::json!({"collections":collections,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(serde_json::json!({"collection":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"handle":r.get::<String,_>("handle"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")}})))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let handle = payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| crate::wizard::slugify(title));
    let r = sqlx::query("INSERT INTO product_collections (id, title, handle, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW()) RETURNING id, title, handle, metadata, created_at, updated_at")
        .bind(id).bind(title).bind(handle).bind(payload.get("metadata").cloned()).fetch_one(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"collection":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"handle":r.get::<String,_>("handle"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")}}))))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("UPDATE product_collections SET title = COALESCE($2, title), handle = COALESCE($3, handle), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING id, title, handle, metadata, created_at, updated_at")
        .bind(id).bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(serde_json::json!({"collection":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"handle":r.get::<String,_>("handle")}})))
}

pub async fn delete_one(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE product_collections SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":id,"object":"product-collection","deleted":true})))
}

#[derive(Debug, serde::Deserialize)]
pub struct ProductsBatchPayload { pub product_ids: Vec<Uuid> }

pub async fn add_products(State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<ProductsBatchPayload>) -> Result<Json<serde_json::Value>, AppError> {
    for pid in &p.product_ids { sqlx::query("UPDATE products SET collection_id = $1, updated_at = NOW() WHERE id = $2").bind(id).bind(pid).execute(&*state.db).await?; }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn remove_products(State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<ProductsBatchPayload>) -> Result<Json<serde_json::Value>, AppError> {
    for pid in &p.product_ids { sqlx::query("UPDATE products SET collection_id = NULL, updated_at = NOW() WHERE id = $1 AND collection_id = $2").bind(pid).bind(id).execute(&*state.db).await?; }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
