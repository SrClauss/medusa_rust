//! Store gift-cards handler — public lookup by id or code
use axum::{extract::{Path, State}, Json};
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

fn build_gift_card(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "code": r.get::<String, _>("code"),
        "value": r.get::<i64, _>("value"),
        "balance": r.get::<i64, _>("balance"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "is_disabled": r.get::<bool, _>("is_disabled"),
        "ends_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ends_at"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

/// GET /store/gift-cards/:id_or_code
/// Looks up a gift card by UUID or by code string.
pub async fn get(
    State(state): State<AppState>,
    Path(id_or_code): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Try UUID parse first; fall back to code lookup
    let row = if let Ok(id) = id_or_code.parse::<Uuid>() {
        sqlx::query(
            "SELECT id, code, value, balance, region_id, is_disabled, ends_at, \
             created_at, updated_at \
             FROM gift_cards WHERE id = $1 AND deleted_at IS NULL AND is_disabled = FALSE",
        )
        .bind(id)
        .fetch_optional(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, code, value, balance, region_id, is_disabled, ends_at, \
             created_at, updated_at \
             FROM gift_cards WHERE code = $1 AND deleted_at IS NULL AND is_disabled = FALSE",
        )
        .bind(&id_or_code)
        .fetch_optional(&*state.db)
        .await?
    };
    let r = row.ok_or_else(|| AppError::NotFound("Gift card not found".into()))?;
    Ok(Json(serde_json::json!({ "gift_card": build_gift_card(&r) })))
}
