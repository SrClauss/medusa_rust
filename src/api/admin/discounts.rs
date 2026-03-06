//! Admin discounts handlers — full CRUD with conditions
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
    pub q: Option<String>,
}
fn d20() -> i64 { 20 }

async fn build_discount(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let rule_id: Option<Uuid> = r.get("rule_id");
    let rule = if let Some(rid) = rule_id {
        sqlx::query(
            "SELECT id, description, type, value, allocation, metadata, created_at, updated_at \
             FROM discount_rules WHERE id = $1",
        )
        .bind(rid)
        .fetch_optional(&*state.db)
        .await?
        .map(|rr| serde_json::json!({
            "id": rr.get::<Uuid, _>("id"),
            "description": rr.get::<Option<String>, _>("description"),
            "type": rr.get::<String, _>("type"),
            "value": rr.get::<i64, _>("value"),
            "allocation": rr.get::<Option<String>, _>("allocation"),
        }))
    } else {
        None
    };

    let regions = sqlx::query(
        "SELECT r.id, r.name FROM regions r \
         JOIN discount_regions dr ON dr.region_id = r.id \
         WHERE dr.discount_id = $1 AND r.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| serde_json::json!({ "id": r.get::<Uuid, _>("id"), "name": r.get::<String, _>("name") }))
    .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "code": r.get::<String, _>("code"),
        "is_dynamic": r.get::<bool, _>("is_dynamic"),
        "rule_id": rule_id,
        "is_disabled": r.get::<bool, _>("is_disabled"),
        "starts_at": r.get::<chrono::DateTime<chrono::Utc>, _>("starts_at"),
        "ends_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ends_at"),
        "valid_duration": r.get::<Option<String>, _>("valid_duration"),
        "usage_limit": r.get::<Option<i32>, _>("usage_limit"),
        "usage_count": r.get::<i32, _>("usage_count"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "deleted_at": null,
        "rule": rule,
        "regions": regions,
    }))
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at \
         FROM discounts WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM discounts WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?;
    let mut discounts = Vec::new();
    for r in &rows {
        discounts.push(build_discount(&state, r.get("id"), r).await?);
    }
    Ok(Json(serde_json::json!({ "discounts": discounts, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at \
         FROM discounts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Discount not found".into()))?;
    Ok(Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? })))
}

pub async fn get_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at \
         FROM discounts WHERE code = $1 AND deleted_at IS NULL",
    )
    .bind(&code)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Discount not found".into()))?;
    let id: Uuid = r.get("id");
    Ok(Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let code = payload.get("code").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("code required".into()))?;
    let code = code.to_uppercase();

    let rule_id: Option<Uuid> = if let Some(rule) = payload.get("rule") {
        let rid = Uuid::new_v4();
        let rule_type = rule.get("type").and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("rule.type required".into()))?;
        let value: i64 = rule.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
        sqlx::query(
            "INSERT INTO discount_rules (id, description, type, value, allocation, metadata, created_at, updated_at) \
             VALUES ($1,$2,$3,$4,$5,$6,NOW(),NOW())",
        )
        .bind(rid)
        .bind(rule.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(rule_type)
        .bind(value)
        .bind(rule.get("allocation").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(rule.get("metadata").cloned())
        .execute(&*state.db)
        .await?;
        Some(rid)
    } else {
        None
    };

    let r = sqlx::query(
        "INSERT INTO discounts (id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, \
         valid_duration, usage_limit, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,NOW(),$6,$7,$8,$9,NOW(),NOW()) \
         RETURNING id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(&code)
    .bind(payload.get("is_dynamic").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(rule_id)
    .bind(payload.get("is_disabled").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(payload.get("ends_at").and_then(|v| v.as_str()).and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()))
    .bind(payload.get("valid_duration").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("usage_limit").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;

    if let Some(regions) = payload.get("regions").and_then(|v| v.as_array()) {
        for region_id in regions {
            if let Some(rid) = region_id.as_str().and_then(|s| s.parse::<Uuid>().ok()) {
                if let Err(e) = sqlx::query(
                    "INSERT INTO discount_regions (discount_id, region_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
                )
                .bind(id)
                .bind(rid)
                .execute(&*state.db)
                .await {
                    tracing::warn!("Failed to link region {} to discount {}: {}", rid, id, e);
                }
            }
        }
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE discounts SET \
         is_disabled = COALESCE($2, is_disabled), \
         ends_at = COALESCE($3, ends_at), \
         valid_duration = COALESCE($4, valid_duration), \
         usage_limit = COALESCE($5, usage_limit), \
         metadata = COALESCE($6, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("is_disabled").and_then(|v| v.as_bool()))
    .bind(payload.get("ends_at").and_then(|v| v.as_str()).and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()))
    .bind(payload.get("valid_duration").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("usage_limit").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Discount not found".into()))?;
    Ok(Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE discounts SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "discount", "deleted": true })))
}

pub async fn add_region(
    State(state): State<AppState>,
    Path((id, region_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "INSERT INTO discount_regions (discount_id, region_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(region_id)
    .execute(&*state.db)
    .await?;
    let r = sqlx::query(
        "SELECT id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at \
         FROM discounts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Discount not found".into()))?;
    Ok(Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? })))
}

pub async fn remove_region(
    State(state): State<AppState>,
    Path((id, region_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM discount_regions WHERE discount_id = $1 AND region_id = $2")
        .bind(id)
        .bind(region_id)
        .execute(&*state.db)
        .await?;
    let r = sqlx::query(
        "SELECT id, code, is_dynamic, rule_id, is_disabled, starts_at, ends_at, valid_duration, \
         usage_limit, usage_count, metadata, created_at, updated_at \
         FROM discounts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Discount not found".into()))?;
    Ok(Json(serde_json::json!({ "discount": build_discount(&state, id, &r).await? })))
}

pub async fn list_conditions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rule_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT rule_id FROM discounts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .flatten();

    let conditions = if let Some(rid) = rule_id {
        sqlx::query(
            "SELECT id, type, operator, metadata, created_at, updated_at \
             FROM discount_conditions WHERE discount_rule_id = $1 AND deleted_at IS NULL",
        )
        .bind(rid)
        .fetch_all(&*state.db)
        .await?
        .into_iter()
        .map(|c| serde_json::json!({
            "id": c.get::<Uuid, _>("id"),
            "type": c.get::<String, _>("type"),
            "operator": c.get::<String, _>("operator"),
            "discount_rule_id": rid,
            "metadata": c.get::<Option<serde_json::Value>, _>("metadata"),
            "created_at": c.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "updated_at": c.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        }))
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Ok(Json(serde_json::json!({ "discount_conditions": conditions })))
}

pub async fn create_condition(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let rule_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT rule_id FROM discounts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .flatten();
    let rule_id = rule_id.ok_or_else(|| AppError::BadRequest("Discount has no rule".into()))?;
    let cid = Uuid::new_v4();
    let condition_type = payload.get("type").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("type required".into()))?;
    let operator = payload.get("operator").and_then(|v| v.as_str()).unwrap_or("in");
    let r = sqlx::query(
        "INSERT INTO discount_conditions (id, discount_rule_id, type, operator, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,NOW(),NOW()) \
         RETURNING id, type, operator, metadata, created_at, updated_at",
    )
    .bind(cid)
    .bind(rule_id)
    .bind(condition_type)
    .bind(operator)
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "discount_condition": {
        "id": r.get::<Uuid, _>("id"),
        "type": r.get::<String, _>("type"),
        "operator": r.get::<String, _>("operator"),
        "discount_rule_id": rule_id,
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }}))))
}

pub async fn get_condition(
    State(state): State<AppState>,
    Path((_id, cid)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, discount_rule_id, type, operator, metadata, created_at, updated_at \
         FROM discount_conditions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(cid)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Condition not found".into()))?;
    Ok(Json(serde_json::json!({ "discount_condition": {
        "id": r.get::<Uuid, _>("id"),
        "type": r.get::<String, _>("type"),
        "operator": r.get::<String, _>("operator"),
        "discount_rule_id": r.get::<Uuid, _>("discount_rule_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }})))
}

pub async fn update_condition(
    State(state): State<AppState>,
    Path((_id, cid)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE discount_conditions SET \
         operator = COALESCE($2, operator), \
         metadata = COALESCE($3, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, discount_rule_id, type, operator, metadata, created_at, updated_at",
    )
    .bind(cid)
    .bind(payload.get("operator").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Condition not found".into()))?;
    Ok(Json(serde_json::json!({ "discount_condition": {
        "id": r.get::<Uuid, _>("id"),
        "type": r.get::<String, _>("type"),
        "operator": r.get::<String, _>("operator"),
        "discount_rule_id": r.get::<Uuid, _>("discount_rule_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }})))
}

pub async fn delete_condition(
    State(state): State<AppState>,
    Path((_id, cid)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE discount_conditions SET deleted_at = NOW() WHERE id = $1")
        .bind(cid)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": cid, "object": "discount-condition", "deleted": true })))
}
