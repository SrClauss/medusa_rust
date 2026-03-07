//! Store shipping-options handlers
use axum::extract::{Path, State};
use axum::Json;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

fn opt_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "profile_id": r.get::<Uuid, _>("profile_id"),
        "provider_id": r.get::<String, _>("provider_id"),
        "price_type": r.get::<String, _>("price_type"),
        "amount": r.get::<Option<i64>, _>("amount"),
        "is_return": r.get::<bool, _>("is_return"),
        "admin_only": r.get::<bool, _>("admin_only"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "requirements": [],
    })
}

pub async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, name, region_id, profile_id, provider_id, price_type, amount, is_return, admin_only, metadata, created_at, updated_at FROM shipping_options WHERE deleted_at IS NULL AND admin_only = false AND is_return = false")
        .fetch_all(&*state.db).await?;
    let options: Vec<_> = rows.iter().map(|r| opt_json(r)).collect();
    Ok(Json(serde_json::json!({ "shipping_options": options })))
}

pub async fn get_for_cart(State(state): State<AppState>, Path(cart_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cart = sqlx::query("SELECT region_id FROM carts WHERE id = $1")
        .bind(cart_id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Cart not found".into()))?;
    let region_id: Uuid = cart.get("region_id");
    let rows = sqlx::query("SELECT id, name, region_id, profile_id, provider_id, price_type, amount, is_return, admin_only, metadata, created_at, updated_at FROM shipping_options WHERE region_id = $1 AND deleted_at IS NULL AND admin_only = false AND is_return = false")
        .bind(region_id).fetch_all(&*state.db).await?;
    let options: Vec<_> = rows.iter().map(|r| opt_json(r)).collect();
    Ok(Json(serde_json::json!({ "shipping_options": options })))
}

// ─── Shipping option calculate (stub) ─────────────────────────────────────────

pub async fn calculate(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({ "shipping_option": { "id": id, "amount": 0 } })))
}
