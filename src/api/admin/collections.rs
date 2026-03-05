//! Admin collection handlers
//! GET/POST /admin/collections
//! GET/PUT/DELETE /admin/collections/:id
//! POST/DELETE /admin/collections/:id/products/batch

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

#[derive(Debug, Serialize)]
pub struct CollectionsResponse { pub collections: Vec<serde_json::Value>, pub count: i64, pub offset: i64, pub limit: i64 }
#[derive(Debug, Serialize)]
pub struct CollectionResponse { pub collection: serde_json::Value }
#[derive(Debug, Serialize)]
pub struct DeleteResponse { pub id: Uuid, pub object: &'static str, pub deleted: bool }

fn map_collection(r: &impl CollectionRow) -> serde_json::Value {
    serde_json::json!({ "id": r.id(), "title": r.title(), "handle": r.handle(), "metadata": r.meta(), "created_at": r.created_at(), "updated_at": r.updated_at(), "deleted_at": null, "products": [] })
}

trait CollectionRow {
    fn id(&self) -> Uuid;
    fn title(&self) -> &str;
    fn handle(&self) -> &str;
    fn meta(&self) -> Option<&serde_json::Value>;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<CollectionsResponse>, AppError> {
    let rows = sqlx::query!("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2", p.limit, p.offset)
        .fetch_all(&*state.db).await?;
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM product_collections WHERE deleted_at IS NULL").fetch_one(&*state.db).await?.unwrap_or(0);
    let collections = rows.iter().map(|r| serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle, "metadata": r.metadata, "created_at": r.created_at, "updated_at": r.updated_at, "deleted_at": null, "products": [] })).collect();
    Ok(Json(CollectionsResponse { collections, count, offset: p.offset, limit: p.limit }))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CollectionResponse>, AppError> {
    let r = sqlx::query!("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE id = $1 AND deleted_at IS NULL", id)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(CollectionResponse { collection: serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle, "metadata": r.metadata, "created_at": r.created_at, "updated_at": r.updated_at, "deleted_at": null }) }))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<CollectionResponse>), AppError> {
    let id = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let handle = payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| crate::wizard::slugify(title));
    let r = sqlx::query!("INSERT INTO product_collections (id, title, handle, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW()) RETURNING id, title, handle, metadata, created_at, updated_at", id, title, handle, payload.get("metadata").cloned())
        .fetch_one(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(CollectionResponse { collection: serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle, "metadata": r.metadata, "created_at": r.created_at, "updated_at": r.updated_at, "deleted_at": null }) })))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<CollectionResponse>, AppError> {
    let r = sqlx::query!("UPDATE product_collections SET title = COALESCE($2, title), handle = COALESCE($3, handle), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING id, title, handle, metadata, created_at, updated_at",
        id,
        payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()),
    ).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(CollectionResponse { collection: serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle, "metadata": r.metadata, "created_at": r.created_at, "updated_at": r.updated_at }) }))
}

pub async fn delete_one(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<DeleteResponse>, AppError> {
    sqlx::query!("UPDATE product_collections SET deleted_at = NOW() WHERE id = $1", id).execute(&*state.db).await?;
    Ok(Json(DeleteResponse { id, object: "product-collection", deleted: true }))
}

#[derive(Debug, Deserialize)]
pub struct ProductsBatchPayload { pub product_ids: Vec<Uuid> }

pub async fn add_products(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<ProductsBatchPayload>) -> Result<Json<CollectionResponse>, AppError> {
    for pid in &payload.product_ids {
        sqlx::query!("UPDATE products SET collection_id = $1, updated_at = NOW() WHERE id = $2", id, pid).execute(&*state.db).await?;
    }
    let r = sqlx::query!("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE id = $1", id)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(CollectionResponse { collection: serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle }) }))
}

pub async fn remove_products(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<ProductsBatchPayload>) -> Result<Json<CollectionResponse>, AppError> {
    for pid in &payload.product_ids {
        sqlx::query!("UPDATE products SET collection_id = NULL, updated_at = NOW() WHERE id = $1 AND collection_id = $2", pid, id).execute(&*state.db).await?;
    }
    let r = sqlx::query!("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE id = $1", id)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(CollectionResponse { collection: serde_json::json!({ "id": r.id, "title": r.title, "handle": r.handle }) }))
}
