//! Store product-categories handlers
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")] pub limit: i64,
    #[serde(default)] pub offset: i64,
}
fn d20() -> i64 { 20 }

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, name, description, handle, is_active, is_internal, parent_category_id, rank, metadata, created_at, updated_at FROM product_categories WHERE is_internal = false AND is_active = true ORDER BY rank, name LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM product_categories WHERE is_internal = false AND is_active = true")
        .fetch_one(&*state.db).await?;
    let categories: Vec<_> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.get::<Option<String>, _>("description"),
        "handle": r.get::<String, _>("handle"),
        "is_active": r.get::<bool, _>("is_active"),
        "is_internal": r.get::<bool, _>("is_internal"),
        "parent_category_id": r.get::<Option<Uuid>, _>("parent_category_id"),
        "rank": r.get::<i32, _>("rank"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "category_children": [],
    })).collect();
    Ok(Json(serde_json::json!({ "product_categories": categories, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, name, description, handle, is_active, is_internal, parent_category_id, rank, metadata, created_at, updated_at FROM product_categories WHERE id = $1")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Category not found".into()))?;
    Ok(Json(serde_json::json!({ "product_category": {
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.get::<Option<String>, _>("description"),
        "handle": r.get::<String, _>("handle"),
        "is_active": r.get::<bool, _>("is_active"),
        "is_internal": r.get::<bool, _>("is_internal"),
        "parent_category_id": r.get::<Option<Uuid>, _>("parent_category_id"),
        "rank": r.get::<i32, _>("rank"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "category_children": [],
        "products": [],
    }})))
}
