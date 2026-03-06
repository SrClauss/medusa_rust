//! Admin regions handlers — full CRUD with countries and providers
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

async fn build_region(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let countries = sqlx::query(
        "SELECT id, iso_2, iso_3, num_code, name, display_name FROM countries WHERE region_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|c| serde_json::json!({
        "id": c.get::<Uuid, _>("id"),
        "iso_2": c.get::<String, _>("iso_2"),
        "iso_3": c.get::<String, _>("iso_3"),
        "num_code": c.get::<Option<i32>, _>("num_code"),
        "name": c.get::<String, _>("name"),
        "display_name": c.get::<String, _>("display_name"),
        "region_id": id,
    }))
    .collect::<Vec<_>>();

    let payment_providers = sqlx::query(
        "SELECT pp.id, pp.is_installed FROM payment_providers pp \
         JOIN region_payment_providers rpp ON rpp.provider_id = pp.id \
         WHERE rpp.region_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|p| serde_json::json!({ "id": p.get::<String, _>("id"), "is_installed": p.get::<bool, _>("is_installed") }))
    .collect::<Vec<_>>();

    let fulfillment_providers = sqlx::query(
        "SELECT fp.id, fp.is_installed FROM fulfillment_providers fp \
         JOIN region_fulfillment_providers rfp ON rfp.provider_id = fp.id \
         WHERE rfp.region_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|p| serde_json::json!({ "id": p.get::<String, _>("id"), "is_installed": p.get::<bool, _>("is_installed") }))
    .collect::<Vec<_>>();

    let tax_rates = sqlx::query(
        "SELECT id, rate, code, name, created_at, updated_at FROM tax_rates WHERE region_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?
    .into_iter()
    .map(|t| serde_json::json!({
        "id": t.get::<Uuid, _>("id"),
        "rate": t.get::<Option<f64>, _>("rate"),
        "code": t.get::<Option<String>, _>("code"),
        "name": t.get::<String, _>("name"),
        "region_id": id,
        "created_at": t.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": t.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }))
    .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "currency_code": r.get::<String, _>("currency_code"),
        "tax_rate": r.get::<f64, _>("tax_rate"),
        "tax_code": r.get::<Option<String>, _>("tax_code"),
        "gift_cards_taxable": r.get::<bool, _>("gift_cards_taxable"),
        "automatic_taxes": r.get::<bool, _>("automatic_taxes"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "countries": countries,
        "payment_providers": payment_providers,
        "fulfillment_providers": fulfillment_providers,
        "tax_rates": tax_rates,
    }))
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at \
         FROM regions WHERE deleted_at IS NULL ORDER BY name LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM regions WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?;
    let mut regions = Vec::new();
    for r in &rows {
        regions.push(build_region(&state, r.get("id"), r).await?);
    }
    Ok(Json(serde_json::json!({ "regions": regions, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at \
         FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let name = payload.get("name").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let currency_code = payload.get("currency_code").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("currency_code required".into()))?;
    let tax_rate: f64 = payload.get("tax_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let r = sqlx::query(
        "INSERT INTO regions (id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, \
         automatic_taxes, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW(),NOW()) \
         RETURNING id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, \
         automatic_taxes, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(currency_code)
    .bind(tax_rate)
    .bind(payload.get("tax_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("gift_cards_taxable").and_then(|v| v.as_bool()).unwrap_or(true))
    .bind(payload.get("automatic_taxes").and_then(|v| v.as_bool()).unwrap_or(true))
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    // Wire up payment providers
    if let Some(payment_providers) = payload.get("payment_providers").and_then(|v| v.as_array()) {
        for pp in payment_providers {
            if let Some(pid) = pp.as_str() {
                if let Err(e) = sqlx::query(
                    "INSERT INTO payment_providers (id, is_installed) VALUES ($1, true) ON CONFLICT DO NOTHING",
                )
                .bind(pid)
                .execute(&*state.db)
                .await {
                    tracing::warn!("Failed to upsert payment_provider {}: {}", pid, e);
                }
                if let Err(e) = sqlx::query(
                    "INSERT INTO region_payment_providers (region_id, provider_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
                )
                .bind(id)
                .bind(pid)
                .execute(&*state.db)
                .await {
                    tracing::warn!("Failed to link payment_provider {} to region {}: {}", pid, id, e);
                }
            }
        }
    }
    // Wire up fulfillment providers
    if let Some(fulfillment_providers) = payload.get("fulfillment_providers").and_then(|v| v.as_array()) {
        for fp in fulfillment_providers {
            if let Some(pid) = fp.as_str() {
                if let Err(e) = sqlx::query(
                    "INSERT INTO fulfillment_providers (id, is_installed) VALUES ($1, true) ON CONFLICT DO NOTHING",
                )
                .bind(pid)
                .execute(&*state.db)
                .await {
                    tracing::warn!("Failed to upsert fulfillment_provider {}: {}", pid, e);
                }
                if let Err(e) = sqlx::query(
                    "INSERT INTO region_fulfillment_providers (region_id, provider_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
                )
                .bind(id)
                .bind(pid)
                .execute(&*state.db)
                .await {
                    tracing::warn!("Failed to link fulfillment_provider {} to region {}: {}", pid, id, e);
                }
            }
        }
    }
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "region": build_region(&state, id, &r).await? }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tax_rate: Option<f64> = payload.get("tax_rate").and_then(|v| v.as_f64());
    let r = sqlx::query(
        "UPDATE regions SET \
         name = COALESCE($2, name), \
         currency_code = COALESCE($3, currency_code), \
         tax_rate = COALESCE($4, tax_rate), \
         tax_code = COALESCE($5, tax_code), \
         metadata = COALESCE($6, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, \
         automatic_taxes, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("currency_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(tax_rate)
    .bind(payload.get("tax_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE regions SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "region", "deleted": true })))
}

pub async fn add_country(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let country_code = payload.get("country_code").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("country_code required".into()))?;
    sqlx::query("UPDATE countries SET region_id = $1 WHERE iso_2 = $2")
        .bind(id)
        .bind(country_code)
        .execute(&*state.db)
        .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn remove_country(
    State(state): State<AppState>,
    Path((region_id, code)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE countries SET region_id = NULL WHERE iso_2 = $1 AND region_id = $2")
        .bind(&code)
        .bind(region_id)
        .execute(&*state.db)
        .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(region_id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, region_id, &r).await? })))
}

pub async fn add_payment_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider_id = payload.get("provider_id").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("provider_id required".into()))?;
    sqlx::query(
        "INSERT INTO payment_providers (id, is_installed) VALUES ($1, true) ON CONFLICT DO NOTHING",
    )
    .bind(provider_id)
    .execute(&*state.db)
    .await?;
    sqlx::query(
        "INSERT INTO region_payment_providers (region_id, provider_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(provider_id)
    .execute(&*state.db)
    .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn remove_payment_provider(
    State(state): State<AppState>,
    Path((id, pid)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "DELETE FROM region_payment_providers WHERE region_id = $1 AND provider_id = $2",
    )
    .bind(id)
    .bind(&pid)
    .execute(&*state.db)
    .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn add_fulfillment_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider_id = payload.get("provider_id").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("provider_id required".into()))?;
    sqlx::query(
        "INSERT INTO fulfillment_providers (id, is_installed) VALUES ($1, true) ON CONFLICT DO NOTHING",
    )
    .bind(provider_id)
    .execute(&*state.db)
    .await?;
    sqlx::query(
        "INSERT INTO region_fulfillment_providers (region_id, provider_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(provider_id)
    .execute(&*state.db)
    .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}

pub async fn remove_fulfillment_provider(
    State(state): State<AppState>,
    Path((id, pid)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "DELETE FROM region_fulfillment_providers WHERE region_id = $1 AND provider_id = $2",
    )
    .bind(id)
    .bind(&pid)
    .execute(&*state.db)
    .await?;
    let r = sqlx::query(
        "SELECT id, name, currency_code, tax_rate, tax_code, gift_cards_taxable, automatic_taxes, \
         metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    Ok(Json(serde_json::json!({ "region": build_region(&state, id, &r).await? })))
}
