//! Admin customer_groups handlers — DB-backed CRUD
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

fn cg_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "customers": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, metadata, created_at, updated_at \
         FROM customer_groups WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM customer_groups WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let customer_groups: Vec<_> = rows.iter().map(cg_json).collect();
    Ok(Json(serde_json::json!({"customer_groups": customer_groups, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, metadata, created_at, updated_at \
         FROM customer_groups WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Customer group not found".into()))?;
    Ok(Json(serde_json::json!({"customer_group": cg_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    sqlx::query(
        "INSERT INTO customer_groups (id, name, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, NOW(), NOW())",
    )
    .bind(id)
    .bind(name)
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
        "UPDATE customer_groups SET \
         name = COALESCE($2, name), \
         metadata = COALESCE($3, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE customer_groups SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "customer-group", "deleted": true})))
}

pub async fn list_customers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT c.id, c.email, c.first_name, c.last_name, c.phone, c.has_account, c.created_at, c.updated_at \
         FROM customers c \
         JOIN customer_group_customers cgc ON cgc.customer_id = c.id \
         WHERE cgc.customer_group_id = $1 AND c.deleted_at IS NULL \
         LIMIT $2 OFFSET $3",
    )
    .bind(id)
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let customers: Vec<_> = rows
        .iter()
        .map(|r| serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "email": r.get::<String, _>("email"),
            "first_name": r.get::<Option<String>, _>("first_name"),
            "last_name": r.get::<Option<String>, _>("last_name"),
            "phone": r.get::<Option<String>, _>("phone"),
            "has_account": r.get::<bool, _>("has_account"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        }))
        .collect();
    Ok(Json(serde_json::json!({"customers": customers, "count": customers.len(), "offset": p.offset, "limit": p.limit})))
}

pub async fn add_customers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(customer_ids) = payload.get("customer_ids").and_then(|v| v.as_array()) {
        for cid_val in customer_ids {
            if let Some(cid_str) = cid_val.as_str() {
                if let Ok(cid) = uuid::Uuid::parse_str(cid_str) {
                    let _ = sqlx::query(
                        "INSERT INTO customer_group_customers (customer_group_id, customer_id) \
                         VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(cid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn remove_customers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(customer_ids) = payload.get("customer_ids").and_then(|v| v.as_array()) {
        for cid_val in customer_ids {
            if let Some(cid_str) = cid_val.as_str() {
                if let Ok(cid) = uuid::Uuid::parse_str(cid_str) {
                    let _ = sqlx::query(
                        "DELETE FROM customer_group_customers WHERE customer_group_id = $1 AND customer_id = $2",
                    )
                    .bind(id)
                    .bind(cid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
