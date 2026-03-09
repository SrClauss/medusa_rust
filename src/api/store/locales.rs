//! Store locales handler — lists available languages from the database.
use axum::{extract::{Query, State}, Json};
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

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT code, name, created_at FROM languages ORDER BY code LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM languages")
        .fetch_one(&*state.db)
        .await?;
    let locales: Vec<_> = rows
        .iter()
        .map(|r| serde_json::json!({
            "code": r.get::<String, _>("code"),
            "name": r.get::<String, _>("name"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        }))
        .collect();
    Ok(Json(serde_json::json!({
        "locales": locales,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}
