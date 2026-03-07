//! Admin inventory handlers — full CRUD with location levels
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
    pub sku: Option<String>,
}
fn d20() -> i64 { 20 }

fn build_item(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "sku": r.get::<Option<String>, _>("sku"),
        "hs_code": r.get::<Option<String>, _>("hs_code"),
        "origin_country": r.get::<Option<String>, _>("origin_country"),
        "mid_code": r.get::<Option<String>, _>("mid_code"),
        "material": r.get::<Option<String>, _>("material"),
        "weight": r.get::<Option<f64>, _>("weight"),
        "length": r.get::<Option<f64>, _>("length"),
        "height": r.get::<Option<f64>, _>("height"),
        "width": r.get::<Option<f64>, _>("width"),
        "requires_shipping": r.get::<bool, _>("requires_shipping"),
        "description": r.get::<Option<String>, _>("description"),
        "thumbnail": r.get::<Option<String>, _>("thumbnail"),
        "title": r.get::<Option<String>, _>("title"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

fn build_level(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "inventory_item_id": r.get::<Uuid, _>("inventory_item_id"),
        "location_id": r.get::<Uuid, _>("location_id"),
        "stocked_quantity": r.get::<i32, _>("stocked_quantity"),
        "reserved_quantity": r.get::<i32, _>("reserved_quantity"),
        "incoming_quantity": r.get::<i32, _>("incoming_quantity"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (rows, count) = if let Some(ref sku) = p.sku {
        let pattern = format!("%{}%", sku);
        let rows = sqlx::query(
            "SELECT id, sku, hs_code, origin_country, mid_code, material, weight, length, \
             height, width, requires_shipping, description, thumbnail, title, metadata, \
             created_at, updated_at \
             FROM inventory_items \
             WHERE deleted_at IS NULL AND sku ILIKE $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?;
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM inventory_items WHERE deleted_at IS NULL AND sku ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&*state.db)
        .await?;
        (rows, count)
    } else {
        let rows = sqlx::query(
            "SELECT id, sku, hs_code, origin_country, mid_code, material, weight, length, \
             height, width, requires_shipping, description, thumbnail, title, metadata, \
             created_at, updated_at \
             FROM inventory_items WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?;
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM inventory_items WHERE deleted_at IS NULL")
                .fetch_one(&*state.db)
                .await?;
        (rows, count)
    };
    let inventory_items: Vec<_> = rows.iter().map(build_item).collect();
    Ok(Json(serde_json::json!({
        "inventory_items": inventory_items,
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
        "SELECT id, sku, hs_code, origin_country, mid_code, material, weight, length, \
         height, width, requires_shipping, description, thumbnail, title, metadata, \
         created_at, updated_at \
         FROM inventory_items WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Inventory item not found".into()))?;

    let levels = sqlx::query(
        "SELECT id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, \
         incoming_quantity, metadata, created_at, updated_at \
         FROM inventory_levels WHERE inventory_item_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|l| build_level(&l))
    .collect::<Vec<_>>();

    let mut item = build_item(&r);
    item["location_levels"] = serde_json::json!(levels);
    Ok(Json(serde_json::json!({ "inventory_item": item })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let r = sqlx::query(
        "INSERT INTO inventory_items \
         (id, sku, hs_code, origin_country, mid_code, material, weight, length, height, width, \
         requires_shipping, description, thumbnail, title, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,NOW(),NOW()) \
         RETURNING id, sku, hs_code, origin_country, mid_code, material, weight, length, \
         height, width, requires_shipping, description, thumbnail, title, metadata, \
         created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(
        payload
            .get("origin_country")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("weight").and_then(|v| v.as_f64()))
    .bind(payload.get("length").and_then(|v| v.as_f64()))
    .bind(payload.get("height").and_then(|v| v.as_f64()))
    .bind(payload.get("width").and_then(|v| v.as_f64()))
    .bind(
        payload
            .get("requires_shipping")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    )
    .bind(
        payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(
        payload
            .get("thumbnail")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    let mut item = build_item(&r);
    item["location_levels"] = serde_json::json!([]);
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "inventory_item": item }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE inventory_items SET \
         sku = COALESCE($2, sku), \
         hs_code = COALESCE($3, hs_code), \
         origin_country = COALESCE($4, origin_country), \
         mid_code = COALESCE($5, mid_code), \
         material = COALESCE($6, material), \
         weight = COALESCE($7, weight), \
         length = COALESCE($8, length), \
         height = COALESCE($9, height), \
         width = COALESCE($10, width), \
         requires_shipping = COALESCE($11, requires_shipping), \
         description = COALESCE($12, description), \
         thumbnail = COALESCE($13, thumbnail), \
         title = COALESCE($14, title), \
         metadata = COALESCE($15, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, sku, hs_code, origin_country, mid_code, material, weight, length, \
         height, width, requires_shipping, description, thumbnail, title, metadata, \
         created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(
        payload
            .get("origin_country")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("weight").and_then(|v| v.as_f64()))
    .bind(payload.get("length").and_then(|v| v.as_f64()))
    .bind(payload.get("height").and_then(|v| v.as_f64()))
    .bind(payload.get("width").and_then(|v| v.as_f64()))
    .bind(payload.get("requires_shipping").and_then(|v| v.as_bool()))
    .bind(
        payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(
        payload
            .get("thumbnail")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Inventory item not found".into()))?;

    let levels = sqlx::query(
        "SELECT id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, \
         incoming_quantity, metadata, created_at, updated_at \
         FROM inventory_levels WHERE inventory_item_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|l| build_level(&l))
    .collect::<Vec<_>>();

    let mut item = build_item(&r);
    item["location_levels"] = serde_json::json!(levels);
    Ok(Json(serde_json::json!({ "inventory_item": item })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE inventory_items SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(
        serde_json::json!({ "id": id, "object": "inventory-item", "deleted": true }),
    ))
}

pub async fn list_levels(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let levels = sqlx::query(
        "SELECT id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, \
         incoming_quantity, metadata, created_at, updated_at \
         FROM inventory_levels WHERE inventory_item_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|l| build_level(&l))
    .collect::<Vec<_>>();
    Ok(Json(serde_json::json!({
        "inventory_levels": levels,
        "count": levels.len(),
    })))
}

pub async fn create_level(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let location_id: Uuid = payload
        .get("location_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("location_id required".into()))?;
    let stocked_quantity: i32 = payload
        .get("stocked_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let level_id = Uuid::new_v4();
    let r = sqlx::query(
        "INSERT INTO inventory_levels \
         (id, inventory_item_id, location_id, stocked_quantity, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,NOW(),NOW()) \
         RETURNING id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, \
         incoming_quantity, metadata, created_at, updated_at",
    )
    .bind(level_id)
    .bind(id)
    .bind(location_id)
    .bind(stocked_quantity)
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "inventory_level": build_level(&r) })),
    ))
}

pub async fn update_level(
    State(state): State<AppState>,
    Path((item_id, location_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE inventory_levels SET \
         stocked_quantity = COALESCE($3, stocked_quantity), \
         incoming_quantity = COALESCE($4, incoming_quantity), \
         metadata = COALESCE($5, metadata), \
         updated_at = NOW() \
         WHERE inventory_item_id = $1 AND location_id = $2 \
         RETURNING id, inventory_item_id, location_id, stocked_quantity, reserved_quantity, \
         incoming_quantity, metadata, created_at, updated_at",
    )
    .bind(item_id)
    .bind(location_id)
    .bind(
        payload
            .get("stocked_quantity")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
    )
    .bind(
        payload
            .get("incoming_quantity")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
    )
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Inventory level not found".into()))?;
    Ok(Json(serde_json::json!({ "inventory_level": build_level(&r) })))
}

pub async fn delete_level(
    State(state): State<AppState>,
    Path((item_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = sqlx::query(
        "DELETE FROM inventory_levels WHERE inventory_item_id = $1 AND location_id = $2",
    )
    .bind(item_id)
    .bind(location_id)
    .execute(&*state.db)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound("Inventory level not found".into()));
    }
    Ok(Json(serde_json::json!({
        "inventory_item_id": item_id,
        "location_id": location_id,
        "deleted": true,
    })))
}
