//! Admin roles handlers — CRUD and user role assignment
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

fn build_role(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "permissions": r.get::<Vec<String>, _>("permissions"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, permissions, created_at, updated_at \
         FROM roles ORDER BY name LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
        .fetch_one(&*state.db)
        .await?;
    let roles: Vec<_> = rows.iter().map(build_role).collect();
    Ok(Json(serde_json::json!({
        "roles": roles,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let permissions: Vec<String> = payload
        .get("permissions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let r = sqlx::query(
        "INSERT INTO roles (name, permissions) VALUES ($1, $2) \
         RETURNING id, name, permissions, created_at, updated_at",
    )
    .bind(name)
    .bind(&permissions)
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "role": build_role(&r) }))))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, permissions, created_at, updated_at FROM roles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Role not found".into()))?;
    Ok(Json(serde_json::json!({ "role": build_role(&r) })))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE roles SET \
           name        = COALESCE($2, name), \
           permissions = COALESCE($3, permissions), \
           updated_at  = now() \
         WHERE id = $1 \
         RETURNING id, name, permissions, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(
        payload
            .get("permissions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect::<Vec<String>>()),
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Role not found".into()))?;
    Ok(Json(serde_json::json!({ "role": build_role(&r) })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Role not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// POST /admin/users/:user_id/roles — assign a role to a user
pub async fn assign_role_to_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let role_id: Uuid = payload
        .get("role_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("role_id (uuid) required".into()))?;
    let r = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) \
         ON CONFLICT (user_id, role_id) DO NOTHING \
         RETURNING id, user_id, role_id, created_at",
    )
    .bind(user_id)
    .bind(role_id)
    .fetch_optional(&*state.db)
    .await?;
    let body = match r {
        Some(row) => serde_json::json!({
            "user_role": {
                "id": row.get::<Uuid, _>("id"),
                "user_id": row.get::<Uuid, _>("user_id"),
                "role_id": row.get::<Uuid, _>("role_id"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            }
        }),
        None => serde_json::json!({ "user_role": null, "message": "already assigned" }),
    };
    Ok((StatusCode::CREATED, Json(body)))
}

/// DELETE /admin/users/:user_id/roles/:role_id — remove a role from a user
pub async fn remove_role_from_user(
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(&*state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /admin/users/:user_id/roles — list roles for a user
pub async fn list_user_roles(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT r.id, r.name, r.permissions, r.created_at, r.updated_at \
         FROM roles r \
         JOIN user_roles ur ON ur.role_id = r.id \
         WHERE ur.user_id = $1 \
         ORDER BY r.name",
    )
    .bind(user_id)
    .fetch_all(&*state.db)
    .await?;
    let roles: Vec<_> = rows.iter().map(build_role).collect();
    Ok(Json(serde_json::json!({ "roles": roles })))
}
