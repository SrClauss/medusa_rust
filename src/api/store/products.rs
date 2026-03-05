//! Store product handlers — GET /store/products[/:id]
use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")] pub limit: i64,
    #[serde(default)] pub offset: i64,
    pub handle: Option<String>,
}
fn d20() -> i64 { 20 }

pub async fn list_products(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE deleted_at IS NULL AND status = 'published' ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND status = 'published'")
        .fetch_one(&*state.db).await?;
    let mut products = Vec::new();
    for r in &rows {
        let id: Uuid = r.get("id");
        products.push(build_product_json(&state, id, r).await?);
    }
    Ok(Json(serde_json::json!({"products":products,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get_product(State(state): State<AppState>, Path(id_or_handle): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let r = if let Ok(uid) = id_or_handle.parse::<Uuid>() {
        sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE id = $1 AND deleted_at IS NULL")
            .bind(uid).fetch_optional(&*state.db).await?
    } else {
        sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE handle = $1 AND deleted_at IS NULL")
            .bind(&id_or_handle).fetch_optional(&*state.db).await?
    }.ok_or_else(|| AppError::NotFound("Product not found".into()))?;
    let id: Uuid = r.get("id");
    Ok(Json(serde_json::json!({"product":build_product_json(&state, id, &r).await?})))
}

async fn build_product_json(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let variants = fetch_variants(state, id).await?;
    let options  = fetch_options(state, id).await?;
    let images   = fetch_images(state, id).await?;
    let col_id: Option<Uuid> = r.get("collection_id");
    let collection = if let Some(cid) = col_id {
        sqlx::query("SELECT id, title, handle, created_at, updated_at FROM product_collections WHERE id = $1")
            .bind(cid).fetch_optional(&*state.db).await?
            .map(|c| serde_json::json!({"id":c.get::<Uuid,_>("id"),"title":c.get::<String,_>("title"),"handle":c.get::<String,_>("handle"),"created_at":c.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":c.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")}))
    } else { None };
    Ok(serde_json::json!({
        "id":r.get::<Uuid,_>("id"),"title":r.get::<String,_>("title"),
        "subtitle":r.get::<Option<String>,_>("subtitle"),"description":r.get::<Option<String>,_>("description"),
        "handle":r.get::<String,_>("handle"),"is_giftcard":r.get::<bool,_>("is_giftcard"),
        "status":r.get::<String,_>("status"),"thumbnail":r.get::<Option<String>,_>("thumbnail"),
        "images":images,"options":options,"variants":variants,
        "collection_id":col_id,"collection":collection,
        "type_id":null,"type":null,"tags":[],"categories":[],
        "discountable":r.get::<bool,_>("discountable"),
        "metadata":r.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"deleted_at":null,
    }))
}

async fn fetch_variants(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, product_id, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, weight, length, height, width, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST")
        .bind(product_id).fetch_all(&*state.db).await?;
    let mut variants = Vec::new();
    for r in rows {
        let vid: Uuid = r.get("id");
        let prices = sqlx::query("SELECT id, currency_code, amount, variant_id, region_id, created_at, updated_at FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL AND price_list_id IS NULL")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|p| serde_json::json!({"id":p.get::<Uuid,_>("id"),"currency_code":p.get::<String,_>("currency_code"),"amount":p.get::<i64,_>("amount"),"variant_id":p.get::<Uuid,_>("variant_id"),"region_id":p.get::<Option<Uuid>,_>("region_id")})).collect::<Vec<_>>();
        let options = sqlx::query("SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE variant_id = $1")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"value":o.get::<String,_>("value"),"option_id":o.get::<Uuid,_>("option_id"),"variant_id":o.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
        variants.push(serde_json::json!({"id":vid,"title":r.get::<String,_>("title"),"product_id":r.get::<Uuid,_>("product_id"),"sku":r.get::<Option<String>,_>("sku"),"barcode":r.get::<Option<String>,_>("barcode"),"ean":r.get::<Option<String>,_>("ean"),"upc":r.get::<Option<String>,_>("upc"),"inventory_quantity":r.get::<i32,_>("inventory_quantity"),"allow_backorder":r.get::<bool,_>("allow_backorder"),"manage_inventory":r.get::<bool,_>("manage_inventory"),"weight":r.get::<Option<f64>,_>("weight"),"variant_rank":r.get::<Option<i32>,_>("variant_rank"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"deleted_at":null,"prices":prices,"options":options}));
    }
    Ok(variants)
}

async fn fetch_options(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, product_id, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL")
        .bind(product_id).fetch_all(&*state.db).await?;
    let mut opts = Vec::new();
    for r in rows {
        let oid: Uuid = r.get("id");
        let values = sqlx::query("SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE option_id = $1")
            .bind(oid).fetch_all(&*state.db).await?
            .into_iter().map(|v| serde_json::json!({"id":v.get::<Uuid,_>("id"),"value":v.get::<String,_>("value"),"option_id":v.get::<Uuid,_>("option_id"),"variant_id":v.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
        opts.push(serde_json::json!({"id":oid,"title":r.get::<String,_>("title"),"product_id":r.get::<Uuid,_>("product_id"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"values":values}));
    }
    Ok(opts)
}

async fn fetch_images(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT pi.id, pi.url, pi.metadata, pi.created_at, pi.updated_at FROM product_images pi JOIN product_images_products pip ON pip.image_id = pi.id WHERE pip.product_id = $1 AND pi.deleted_at IS NULL")
        .bind(product_id).fetch_all(&*state.db).await?;
    Ok(rows.into_iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"url":r.get::<String,_>("url"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect())
}
