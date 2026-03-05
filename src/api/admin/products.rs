//! Admin product handlers (full CRUD)
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

async fn build_product(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let variants = sqlx::query("SELECT id, title, sku, inventory_quantity, allow_backorder, manage_inventory, unit_price, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|v| serde_json::json!({"id":v.get::<Uuid,_>("id"),"title":v.get::<String,_>("title"),"sku":v.get::<Option<String>,_>("sku"),"inventory_quantity":v.get::<i32,_>("inventory_quantity"),"allow_backorder":v.get::<bool,_>("allow_backorder"),"manage_inventory":v.get::<bool,_>("manage_inventory"),"variant_rank":v.get::<Option<i32>,_>("variant_rank"),"metadata":v.get::<Option<serde_json::Value>,_>("metadata"),"created_at":v.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":v.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"prices":[],"options":[]})).collect::<Vec<_>>();
    let options = sqlx::query("SELECT id, title, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"title":o.get::<String,_>("title"),"product_id":id,"metadata":o.get::<Option<serde_json::Value>,_>("metadata"),"created_at":o.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":o.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"values":[]})).collect::<Vec<_>>();
    let images = sqlx::query("SELECT pi.id, pi.url, pi.metadata, pi.created_at, pi.updated_at FROM product_images pi JOIN product_images_products pip ON pip.image_id = pi.id WHERE pip.product_id = $1")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|i| serde_json::json!({"id":i.get::<Uuid,_>("id"),"url":i.get::<String,_>("url")})).collect::<Vec<_>>();
    Ok(serde_json::json!({"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"subtitle":r.get::<Option<String>,_>("subtitle"),"description":r.get::<Option<String>,_>("description"),"handle":r.get::<String,_>("handle"),"is_giftcard":r.get::<bool,_>("is_giftcard"),"status":r.get::<String,_>("status"),"thumbnail":r.get::<Option<String>,_>("thumbnail"),"collection_id":r.get::<Option<Uuid>,_>("collection_id"),"discountable":r.get::<bool,_>("discountable"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"deleted_at":null,"variants":variants,"options":options,"images":images,"tags":[],"categories":[],"type":null,"collection":null}))
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE deleted_at IS NULL").fetch_one(&*state.db).await?;
    let mut products = Vec::new();
    for r in &rows { products.push(build_product(&state, r.get("id"), r).await?); }
    Ok(Json(serde_json::json!({"products":products,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Product not found".into()))?;
    Ok(Json(serde_json::json!({"product":build_product(&state, id, &r).await?})))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let handle = payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| crate::wizard::slugify(title));
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("draft");
    let r = sqlx::query("INSERT INTO products (id, title, subtitle, description, handle, is_giftcard, status, thumbnail, discountable, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,NOW(),NOW()) RETURNING id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at")
        .bind(id).bind(title).bind(payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(handle).bind(false).bind(status).bind(payload.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("discountable").and_then(|v| v.as_bool()).unwrap_or(true)).bind(payload.get("metadata").cloned())
        .fetch_one(&*state.db).await?;
    // Create options and variants if provided
    if let Some(options) = payload.get("options").and_then(|v| v.as_array()) {
        for opt in options {
            let oid = Uuid::new_v4();
            let opt_title = opt.get("title").and_then(|v| v.as_str()).unwrap_or("Option");
            sqlx::query("INSERT INTO product_options (id, title, product_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())").bind(oid).bind(opt_title).bind(id).execute(&*state.db).await?;
        }
    }
    if let Some(variants) = payload.get("variants").and_then(|v| v.as_array()) {
        for (rank, var) in variants.iter().enumerate() {
            let vid = Uuid::new_v4();
            let var_title = var.get("title").and_then(|v| v.as_str()).unwrap_or("Default");
            let inv: i32 = var.get("inventory_quantity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            sqlx::query("INSERT INTO product_variants (id, product_id, title, sku, inventory_quantity, allow_backorder, manage_inventory, variant_rank, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,false,true,$6,NOW(),NOW())").bind(vid).bind(id).bind(var_title).bind(var.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(inv).bind(rank as i32).execute(&*state.db).await?;
            if let Some(prices) = var.get("prices").and_then(|v| v.as_array()) {
                for price in prices {
                    let pid = Uuid::new_v4();
                    let amount: i64 = price.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cc = price.get("currency_code").and_then(|v| v.as_str()).unwrap_or("usd");
                    sqlx::query("INSERT INTO money_amounts (id, currency_code, amount, variant_id, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW())").bind(pid).bind(cc).bind(amount).bind(vid).execute(&*state.db).await?;
                }
            }
        }
    }
    Ok((StatusCode::CREATED, Json(serde_json::json!({"product":build_product(&state, id, &r).await?}))))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE products SET title=COALESCE($2,title), description=COALESCE($3,description), status=COALESCE($4,status), thumbnail=COALESCE($5,thumbnail), updated_at=NOW() WHERE id=$1 AND deleted_at IS NULL")
        .bind(id).bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE products SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":id,"object":"product","deleted":true})))
}

// ─── Variants ─────────────────────────────────────────────────────────────────
pub async fn list_variants(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, sku, inventory_quantity, allow_backorder, manage_inventory, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST").bind(id).fetch_all(&*state.db).await?;
    let variants: Vec<_> = rows.iter().map(|v| serde_json::json!({"id":v.get::<Uuid,_>("id"),"title":v.get::<String,_>("title"),"sku":v.get::<Option<String>,_>("sku"),"product_id":id,"inventory_quantity":v.get::<i32,_>("inventory_quantity"),"prices":[]})).collect();
    let count = variants.len() as i64;
    Ok(Json(serde_json::json!({"variants":variants,"count":count})))
}

pub async fn create_variant(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let vid = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let inv: i32 = payload.get("inventory_quantity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    sqlx::query("INSERT INTO product_variants (id, product_id, title, sku, inventory_quantity, allow_backorder, manage_inventory, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,false,true,NOW(),NOW())").bind(vid).bind(id).bind(title).bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(inv).execute(&*state.db).await?;
    let r = sqlx::query("SELECT id, title, sku, inventory_quantity, allow_backorder, manage_inventory, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE id = $1").bind(vid).fetch_one(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"variant":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"product_id":id,"sku":r.get::<Option<String>,_>("sku"),"inventory_quantity":r.get::<i32,_>("inventory_quantity"),"prices":[],"options":[]}}))))
}

pub async fn update_variant(State(state): State<AppState>, Path((pid, vid)): Path<(Uuid,Uuid)>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let inv: Option<i32> = payload.get("inventory_quantity").and_then(|v| v.as_i64()).map(|v| v as i32);
    sqlx::query("UPDATE product_variants SET title=COALESCE($3,title), sku=COALESCE($4,sku), inventory_quantity=COALESCE($5,inventory_quantity), updated_at=NOW() WHERE id=$1 AND product_id=$2")
        .bind(vid).bind(pid).bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(inv).execute(&*state.db).await?;
    let r = sqlx::query("SELECT id, title, sku, inventory_quantity, allow_backorder, manage_inventory, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE id = $1").bind(vid).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Variant not found".into()))?;
    Ok(Json(serde_json::json!({"variant":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"product_id":pid,"sku":r.get::<Option<String>,_>("sku"),"inventory_quantity":r.get::<i32,_>("inventory_quantity")}})))
}

pub async fn delete_variant(State(state): State<AppState>, Path((_pid, vid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE product_variants SET deleted_at = NOW() WHERE id = $1").bind(vid).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":vid,"object":"product-variant","deleted":true})))
}

pub async fn get_variant(State(state): State<AppState>, Path((_pid, vid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, sku, inventory_quantity, allow_backorder, manage_inventory, product_id, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE id = $1 AND deleted_at IS NULL").bind(vid).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Variant not found".into()))?;
    Ok(Json(serde_json::json!({"variant":{"id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),"product_id":r.get::<Uuid,_>("product_id"),"sku":r.get::<Option<String>,_>("sku"),"inventory_quantity":r.get::<i32,_>("inventory_quantity"),"prices":[],"options":[]}})))
}

// ─── Options ──────────────────────────────────────────────────────────────────
pub async fn list_options(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL").bind(id).fetch_all(&*state.db).await?;
    let options: Vec<_> = rows.iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"title":o.get::<String,_>("title"),"product_id":id})).collect();
    let count = options.len() as i64;
    Ok(Json(serde_json::json!({"options":options,"count":count})))
}

pub async fn create_option(State(state): State<AppState>, Path(pid): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let oid = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    sqlx::query("INSERT INTO product_options (id, title, product_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())").bind(oid).bind(title).bind(pid).execute(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"product_option":{"id":oid,"title":title,"product_id":pid}}))))
}

pub async fn update_option(State(state): State<AppState>, Path((pid, oid)): Path<(Uuid,Uuid)>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE product_options SET title=COALESCE($3,title), updated_at=NOW() WHERE id=$1 AND product_id=$2").bind(oid).bind(pid).bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"product_option":{"id":oid,"product_id":pid}})))
}

pub async fn delete_option(State(state): State<AppState>, Path((_pid, oid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE product_options SET deleted_at = NOW() WHERE id = $1").bind(oid).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":oid,"object":"product-option","deleted":true})))
}
