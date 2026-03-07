//! Store product-tags handlers — public read-only
use axum::{extract::{Path, Query, State}, Json};
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
    pub value: Option<String>,
}
fn d20() -> i64 { 20 }

fn build_tag(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "value": r.get::<String, _>("value"),
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
        "SELECT id, value, metadata, created_at, updated_at \
         FROM product_tags WHERE deleted_at IS NULL \
         AND ($3::text IS NULL OR value ILIKE '%' || REPLACE(REPLACE($3, '%', '\\%'), '_', '\\_') || '%') \
         ORDER BY value ASC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .bind(&p.value)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM product_tags WHERE deleted_at IS NULL \
         AND ($1::text IS NULL OR value ILIKE '%' || REPLACE(REPLACE($1, '%', '\\%'), '_', '\\_') || '%')",
    )
    .bind(&p.value)
    .fetch_one(&*state.db)
    .await?;
    let product_tags: Vec<_> = rows.iter().map(build_tag).collect();
    Ok(Json(serde_json::json!({
        "product_tags": product_tags,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, value, metadata, created_at, updated_at \
         FROM product_tags WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Product tag not found".into()))?;
    Ok(Json(serde_json::json!({ "product_tag": build_tag(&r) })))
}
