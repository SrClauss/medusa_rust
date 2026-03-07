//! Admin promotions handlers — DB-backed CRUD
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

fn promo_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "code": r.get::<String, _>("code"),
        "is_automatic": r.get::<bool, _>("is_automatic"),
        "type": r.get::<String, _>("type"),
        "status": r.get::<String, _>("status"),
        "campaign_id": r.get::<Option<Uuid>, _>("campaign_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "rules": [],
        "application_method": null,
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = if let Some(ref q) = p.q {
        let pattern = format!("%{}%", q.to_lowercase());
        sqlx::query(
            "SELECT id, code, is_automatic, type, status, campaign_id, metadata, created_at, updated_at \
             FROM promotions WHERE deleted_at IS NULL AND LOWER(code) LIKE $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, code, is_automatic, type, status, campaign_id, metadata, created_at, updated_at \
             FROM promotions WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    };
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM promotions WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let promotions: Vec<_> = rows.iter().map(promo_json).collect();
    Ok(Json(serde_json::json!({"promotions": promotions, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, code, is_automatic, type, status, campaign_id, metadata, created_at, updated_at \
         FROM promotions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Promotion not found".into()))?;
    Ok(Json(serde_json::json!({"promotion": promo_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let code = payload
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("code required".into()))?;
    let promo_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("standard");
    let is_automatic = payload.get("is_automatic").and_then(|v| v.as_bool()).unwrap_or(false);
    sqlx::query(
        "INSERT INTO promotions (id, code, is_automatic, type, status, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, 'draft', $5, NOW(), NOW())",
    )
    .bind(id)
    .bind(code.to_uppercase())
    .bind(is_automatic)
    .bind(promo_type)
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE promotions SET \
         is_automatic = COALESCE($2, is_automatic), \
         status = COALESCE($3, status), \
         metadata = COALESCE($4, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(payload.get("is_automatic").and_then(|v| v.as_bool()))
    .bind(payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE promotions SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "promotion", "deleted": true})))
}

pub async fn add_rules(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(rules) = payload.get("rules").and_then(|v| v.as_array()) {
        for rule in rules {
            let rule_id = Uuid::new_v4();
            let attribute = rule.get("attribute").and_then(|v| v.as_str()).unwrap_or("cart");
            let operator = rule.get("operator").and_then(|v| v.as_str()).unwrap_or("eq");
            sqlx::query(
                "INSERT INTO promotion_rules (id, promotion_id, attribute, operator, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, NOW(), NOW())",
            )
            .bind(rule_id)
            .bind(id)
            .bind(attribute)
            .bind(operator)
            .execute(&*state.db)
            .await?;
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn remove_rules(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(rule_ids) = payload.get("rule_ids").and_then(|v| v.as_array()) {
        for rid_val in rule_ids {
            if let Some(rid_str) = rid_val.as_str() {
                if let Ok(rid) = uuid::Uuid::parse_str(rid_str) {
                    let _ = sqlx::query(
                        "DELETE FROM promotion_rules WHERE id = $1 AND promotion_id = $2",
                    )
                    .bind(rid)
                    .bind(id)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn batch_buy_rules(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "promotion": { "id": id, "buy_rules": [] }
    })))
}

pub async fn batch_target_rules(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "promotion": { "id": id, "target_rules": [] }
    })))
}
