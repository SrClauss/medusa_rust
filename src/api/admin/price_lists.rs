//! Admin price-list handlers — full CRUD with prices
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

async fn build_price_list(
    state: &AppState,
    id: Uuid,
    r: &sqlx::postgres::PgRow,
) -> Result<serde_json::Value, AppError> {
    let prices = sqlx::query(
        "SELECT id, currency_code, amount, variant_id, region_id, \
         min_quantity, max_quantity, created_at, updated_at \
         FROM money_amounts WHERE price_list_id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|p| {
        serde_json::json!({
            "id": p.get::<Uuid, _>("id"),
            "currency_code": p.get::<String, _>("currency_code"),
            "amount": p.get::<i64, _>("amount"),
            "variant_id": p.get::<Uuid, _>("variant_id"),
            "region_id": p.get::<Option<Uuid>, _>("region_id"),
            "min_quantity": p.get::<Option<i32>, _>("min_quantity"),
            "max_quantity": p.get::<Option<i32>, _>("max_quantity"),
            "price_list_id": id,
            "created_at": p.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "updated_at": p.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        })
    })
    .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.get::<String, _>("description"),
        "type": r.get::<String, _>("price_list_type"),
        "status": r.get::<String, _>("status"),
        "starts_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("starts_at"),
        "ends_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ends_at"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "prices": prices,
    }))
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, description, price_list_type, status, starts_at, ends_at, \
         created_at, updated_at \
         FROM price_lists WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM price_lists WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let mut price_lists = Vec::new();
    for r in &rows {
        price_lists.push(build_price_list(&state, r.get("id"), r).await?);
    }
    Ok(Json(serde_json::json!({
        "price_lists": price_lists,
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
        "SELECT id, name, description, price_list_type, status, starts_at, ends_at, \
         created_at, updated_at \
         FROM price_lists WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Price list not found".into()))?;
    Ok(Json(serde_json::json!({ "price_list": build_price_list(&state, id, &r).await? })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let id = Uuid::new_v4();
    let r = sqlx::query(
        "INSERT INTO price_lists \
         (id, name, description, price_list_type, status, starts_at, ends_at, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW()) \
         RETURNING id, name, description, price_list_type, status, starts_at, ends_at, \
         created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(
        payload
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    )
    .bind(payload.get("type").and_then(|v| v.as_str()).unwrap_or("sale"))
    .bind(
        payload
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active"),
    )
    .bind(
        payload
            .get("starts_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .bind(
        payload
            .get("ends_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .fetch_one(&*state.db)
    .await?;

    // Insert prices if provided
    if let Some(prices) = payload.get("prices").and_then(|v| v.as_array()) {
        for price in prices {
            let variant_id: Option<Uuid> = price
                .get("variant_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok());
            let currency_code = price.get("currency_code").and_then(|v| v.as_str());
            let amount = price.get("amount").and_then(|v| v.as_i64());
            if let (Some(vid), Some(cc), Some(amt)) = (variant_id, currency_code, amount) {
                let pid = Uuid::new_v4();
                if let Err(e) = sqlx::query(
                    "INSERT INTO money_amounts \
                     (id, currency_code, amount, variant_id, region_id, price_list_id, \
                     min_quantity, max_quantity, created_at, updated_at) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW(),NOW())",
                )
                .bind(pid)
                .bind(cc)
                .bind(amt)
                .bind(vid)
                .bind(
                    price
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<Uuid>().ok()),
                )
                .bind(id)
                .bind(
                    price
                        .get("min_quantity")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32),
                )
                .bind(
                    price
                        .get("max_quantity")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32),
                )
                .execute(&*state.db)
                .await
                {
                    tracing::warn!("Failed to insert price for price list {}: {}", id, e);
                }
            }
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "price_list": build_price_list(&state, id, &r).await? })),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE price_lists SET \
         name = COALESCE($2, name), \
         description = COALESCE($3, description), \
         price_list_type = COALESCE($4, price_list_type), \
         status = COALESCE($5, status), \
         starts_at = COALESCE($6, starts_at), \
         ends_at = COALESCE($7, ends_at), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, name, description, price_list_type, status, starts_at, ends_at, \
         created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(
        payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(payload.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(
        payload
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    )
    .bind(
        payload
            .get("starts_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .bind(
        payload
            .get("ends_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Price list not found".into()))?;
    Ok(Json(serde_json::json!({ "price_list": build_price_list(&state, id, &r).await? })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE price_lists SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "price-list", "deleted": true })))
}

pub async fn add_prices(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify price list exists
    let r = sqlx::query(
        "SELECT id, name, description, price_list_type, status, starts_at, ends_at, \
         created_at, updated_at \
         FROM price_lists WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Price list not found".into()))?;

    if let Some(prices) = payload.get("prices").and_then(|v| v.as_array()) {
        for price in prices {
            let variant_id: Option<Uuid> = price
                .get("variant_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok());
            let currency_code = price.get("currency_code").and_then(|v| v.as_str());
            let amount = price.get("amount").and_then(|v| v.as_i64());
            if let (Some(vid), Some(cc), Some(amt)) = (variant_id, currency_code, amount) {
                let pid = Uuid::new_v4();
                if let Err(e) = sqlx::query(
                    "INSERT INTO money_amounts \
                     (id, currency_code, amount, variant_id, region_id, price_list_id, \
                     min_quantity, max_quantity, created_at, updated_at) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW(),NOW()) \
                     ON CONFLICT DO NOTHING",
                )
                .bind(pid)
                .bind(cc)
                .bind(amt)
                .bind(vid)
                .bind(
                    price
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<Uuid>().ok()),
                )
                .bind(id)
                .bind(
                    price
                        .get("min_quantity")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32),
                )
                .bind(
                    price
                        .get("max_quantity")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32),
                )
                .execute(&*state.db)
                .await
                {
                    tracing::warn!("Failed to insert price: {}", e);
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "price_list": build_price_list(&state, id, &r).await? })))
}

pub async fn delete_prices(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(price_ids) = payload.get("price_ids").and_then(|v| v.as_array()) {
        let mut deleted_ids: Vec<serde_json::Value> = Vec::new();
        for price_id in price_ids {
            if let Some(pid) = price_id.as_str().and_then(|s| s.parse::<Uuid>().ok()) {
                match sqlx::query(
                    "UPDATE money_amounts SET deleted_at = NOW() \
                     WHERE id = $1 AND price_list_id = $2 AND deleted_at IS NULL",
                )
                .bind(pid)
                .bind(id)
                .execute(&*state.db)
                .await
                {
                    Ok(res) if res.rows_affected() > 0 => {
                        deleted_ids.push(serde_json::json!(pid));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("Failed to delete price {}: {}", pid, e);
                    }
                }
            }
        }
        return Ok(Json(serde_json::json!({ "ids": deleted_ids, "object": "money-amount", "deleted": true })));
    }
    Ok(Json(serde_json::json!({ "ids": [], "object": "money-amount", "deleted": true })))
}

pub async fn list_products(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT p.id) \
         FROM products p \
         JOIN product_variants pv ON pv.product_id = p.id \
         JOIN money_amounts ma ON ma.variant_id = pv.id \
         WHERE ma.price_list_id = $1 AND ma.deleted_at IS NULL AND p.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_one(&*state.db)
    .await?;
    let rows = sqlx::query(
        "SELECT DISTINCT p.id, p.title, p.handle, p.status, p.created_at, p.updated_at \
         FROM products p \
         JOIN product_variants pv ON pv.product_id = p.id \
         JOIN money_amounts ma ON ma.variant_id = pv.id \
         WHERE ma.price_list_id = $1 AND ma.deleted_at IS NULL AND p.deleted_at IS NULL \
         ORDER BY p.created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(id)
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let products: Vec<_> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "title": r.get::<String, _>("title"),
                "handle": r.get::<Option<String>, _>("handle"),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "products": products,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}
