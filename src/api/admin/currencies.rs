//! Admin currencies handlers — list, get, update
use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use sqlx::Row;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn d20() -> i64 { 20 }

fn build_currency(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "code": r.get::<String, _>("code"),
        "symbol": r.get::<String, _>("symbol"),
        "symbol_native": r.get::<String, _>("symbol_native"),
        "name": r.get::<String, _>("name"),
        "includes_tax": r.get::<bool, _>("includes_tax"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT code, symbol, symbol_native, name, includes_tax \
         FROM currencies ORDER BY code LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM currencies")
        .fetch_one(&*state.db)
        .await?;
    let currencies: Vec<_> = rows.iter().map(build_currency).collect();
    Ok(Json(serde_json::json!({
        "currencies": currencies,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT code, symbol, symbol_native, name, includes_tax \
         FROM currencies WHERE code = $1",
    )
    .bind(&code)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Currency not found".into()))?;
    Ok(Json(serde_json::json!({ "currency": build_currency(&r) })))
}

pub async fn update(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let includes_tax = payload
        .get("includes_tax")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::BadRequest("includes_tax required".into()))?;
    let r = sqlx::query(
        "UPDATE currencies SET includes_tax = $2 WHERE code = $1 \
         RETURNING code, symbol, symbol_native, name, includes_tax",
    )
    .bind(&code)
    .bind(includes_tax)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Currency not found".into()))?;
    Ok(Json(serde_json::json!({ "currency": build_currency(&r) })))
}
