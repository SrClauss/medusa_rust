//! Admin languages handlers — CRUD and product translation management
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

fn build_language(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "code": r.get::<String, _>("code"),
        "name": r.get::<String, _>("name"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
    })
}

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
    let languages: Vec<_> = rows.iter().map(build_language).collect();
    Ok(Json(serde_json::json!({
        "languages": languages,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let code = payload
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("code required".into()))?;
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let r = sqlx::query(
        "INSERT INTO languages (code, name) VALUES ($1, $2) \
         RETURNING code, name, created_at",
    )
    .bind(code)
    .bind(name)
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "language": build_language(&r) }))))
}

pub async fn get(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT code, name, created_at FROM languages WHERE code = $1")
        .bind(&code)
        .fetch_optional(&*state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Language not found".into()))?;
    Ok(Json(serde_json::json!({ "language": build_language(&r) })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM languages WHERE code = $1")
        .bind(&code)
        .execute(&*state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Language not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// POST /admin/products/:product_id/translations — upsert a product translation
pub async fn upsert_product_translation(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let language_code = payload
        .get("language_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("language_code required".into()))?;
    let title = payload.get("title").and_then(|v| v.as_str());
    let description = payload.get("description").and_then(|v| v.as_str());
    let r = sqlx::query(
        "INSERT INTO product_translations (product_id, language_code, title, description) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (product_id, language_code) DO UPDATE \
           SET title = EXCLUDED.title, description = EXCLUDED.description \
         RETURNING id, product_id, language_code, title, description",
    )
    .bind(product_id)
    .bind(language_code)
    .bind(title)
    .bind(description)
    .fetch_one(&*state.db)
    .await?;
    let translation = serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "product_id": r.get::<Uuid, _>("product_id"),
        "language_code": r.get::<String, _>("language_code"),
        "title": r.get::<Option<String>, _>("title"),
        "description": r.get::<Option<String>, _>("description"),
    });
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "translation": translation }))))
}

/// GET /admin/products/:product_id/translations — list translations for a product
pub async fn list_product_translations(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, product_id, language_code, title, description \
         FROM product_translations WHERE product_id = $1 ORDER BY language_code",
    )
    .bind(product_id)
    .fetch_all(&*state.db)
    .await?;
    let translations: Vec<_> = rows
        .iter()
        .map(|r| serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "product_id": r.get::<Uuid, _>("product_id"),
            "language_code": r.get::<String, _>("language_code"),
            "title": r.get::<Option<String>, _>("title"),
            "description": r.get::<Option<String>, _>("description"),
        }))
        .collect();
    Ok(Json(serde_json::json!({ "translations": translations })))
}
