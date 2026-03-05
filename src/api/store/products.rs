//! Store product handlers — GET /store/products[/:id]
//! Response format matches MedusaJS v1 storefront expectations.

use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")] pub limit: i64,
    #[serde(default)] pub offset: i64,
    pub q: Option<String>,
    pub handle: Option<String>,
    pub collection_id: Option<Vec<Uuid>>,
    pub category_id: Option<Vec<Uuid>>,
    pub id: Option<Vec<Uuid>>,
    pub is_giftcard: Option<bool>,
    pub order: Option<String>,
    pub expand: Option<String>,
    pub fields: Option<String>,
    pub cart_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub currency_code: Option<String>,
}
fn d20() -> i64 { 20 }

pub async fn list_products(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, title, subtitle, description, handle, is_giftcard, status,
                  thumbnail, weight, length, height, width, hs_code, origin_country,
                  mid_code, material, collection_id, type_id, discountable,
                  external_id, metadata, created_at, updated_at, deleted_at
           FROM products
           WHERE deleted_at IS NULL AND status = 'published'
           ORDER BY created_at DESC
           LIMIT $1 OFFSET $2"#,
        p.limit, p.offset
    )
    .fetch_all(&*state.db).await?;

    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND status = 'published'"
    )
    .fetch_one(&*state.db).await?.unwrap_or(0);

    let mut products = Vec::new();
    for r in &rows {
        let product = build_product_json(&state, r.id, r.title.clone(), r.subtitle.clone(), r.description.clone(), r.handle.clone(), r.is_giftcard, r.status.clone(), r.thumbnail.clone(), r.collection_id, r.discountable, r.metadata.clone(), r.created_at, r.updated_at).await?;
        products.push(product);
    }

    Ok(Json(serde_json::json!({ "products": products, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id_or_handle): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Support both UUID id and handle
    let r = if let Ok(uid) = id_or_handle.parse::<Uuid>() {
        sqlx::query!(
            "SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE id = $1 AND deleted_at IS NULL",
            uid
        ).fetch_optional(&*state.db).await?
    } else {
        sqlx::query!(
            "SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, discountable, metadata, created_at, updated_at FROM products WHERE handle = $1 AND deleted_at IS NULL",
            id_or_handle
        ).fetch_optional(&*state.db).await?
    }.ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    let product = build_product_json(&state, r.id, r.title, r.subtitle, r.description, r.handle, r.is_giftcard, r.status, r.thumbnail, r.collection_id, r.discountable, r.metadata, r.created_at, r.updated_at).await?;
    Ok(Json(serde_json::json!({ "product": product })))
}

async fn build_product_json(
    state: &AppState,
    id: Uuid,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    handle: String,
    is_giftcard: bool,
    status: String,
    thumbnail: Option<String>,
    collection_id: Option<Uuid>,
    discountable: bool,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<serde_json::Value, AppError> {
    // Variants with prices
    let variants = fetch_variants(state, id).await?;
    // Options
    let options = fetch_options(state, id).await?;
    // Images
    let images = fetch_images(state, id).await?;
    // Collection
    let collection = if let Some(cid) = collection_id {
        sqlx::query!("SELECT id, title, handle, created_at, updated_at FROM product_collections WHERE id = $1", cid)
            .fetch_optional(&*state.db).await?
            .map(|c| serde_json::json!({"id":c.id,"title":c.title,"handle":c.handle,"created_at":c.created_at,"updated_at":c.updated_at}))
    } else { None };

    Ok(serde_json::json!({
        "id": id,
        "title": title,
        "subtitle": subtitle,
        "description": description,
        "handle": handle,
        "is_giftcard": is_giftcard,
        "status": status,
        "thumbnail": thumbnail,
        "images": images,
        "options": options,
        "variants": variants,
        "collection_id": collection_id,
        "collection": collection,
        "type_id": null,
        "type": null,
        "tags": [],
        "categories": [],
        "discountable": discountable,
        "metadata": metadata,
        "created_at": created_at,
        "updated_at": updated_at,
        "deleted_at": null,
    }))
}

async fn fetch_variants(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT id, title, product_id, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, weight, length, height, width, variant_rank, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST",
        product_id
    ).fetch_all(&*state.db).await?;

    let mut variants = Vec::new();
    for r in rows {
        let prices = sqlx::query!(
            "SELECT id, currency_code, amount, variant_id, region_id, price_list_id, min_quantity, max_quantity, created_at, updated_at FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL AND price_list_id IS NULL",
            r.id
        ).fetch_all(&*state.db).await?
        .into_iter()
        .map(|p| serde_json::json!({"id":p.id,"currency_code":p.currency_code,"amount":p.amount,"variant_id":p.variant_id,"region_id":p.region_id,"min_quantity":p.min_quantity,"max_quantity":p.max_quantity,"created_at":p.created_at,"updated_at":p.updated_at}))
        .collect::<Vec<_>>();

        let options = sqlx::query!(
            "SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE variant_id = $1",
            r.id
        ).fetch_all(&*state.db).await?
        .into_iter()
        .map(|o| serde_json::json!({"id":o.id,"value":o.value,"option_id":o.option_id,"variant_id":o.variant_id,"created_at":o.created_at,"updated_at":o.updated_at}))
        .collect::<Vec<_>>();

        variants.push(serde_json::json!({
            "id": r.id,
            "title": r.title,
            "product_id": r.product_id,
            "sku": r.sku,
            "barcode": r.barcode,
            "ean": r.ean,
            "upc": r.upc,
            "inventory_quantity": r.inventory_quantity,
            "allow_backorder": r.allow_backorder,
            "manage_inventory": r.manage_inventory,
            "weight": r.weight,
            "length": r.length,
            "height": r.height,
            "width": r.width,
            "variant_rank": r.variant_rank,
            "metadata": r.metadata,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
            "deleted_at": null,
            "prices": prices,
            "options": options,
        }));
    }
    Ok(variants)
}

async fn fetch_options(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT id, title, product_id, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL",
        product_id
    ).fetch_all(&*state.db).await?;

    let mut opts = Vec::new();
    for r in rows {
        let values = sqlx::query!(
            "SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE option_id = $1",
            r.id
        ).fetch_all(&*state.db).await?
        .into_iter()
        .map(|v| serde_json::json!({"id":v.id,"value":v.value,"option_id":v.option_id,"variant_id":v.variant_id,"created_at":v.created_at,"updated_at":v.updated_at}))
        .collect::<Vec<_>>();

        opts.push(serde_json::json!({"id":r.id,"title":r.title,"product_id":r.product_id,"metadata":r.metadata,"created_at":r.created_at,"updated_at":r.updated_at,"values":values}));
    }
    Ok(opts)
}

async fn fetch_images(state: &AppState, product_id: Uuid) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT pi.id, pi.url, pi.metadata, pi.created_at, pi.updated_at FROM product_images pi JOIN product_images_products pip ON pip.image_id = pi.id WHERE pip.product_id = $1 AND pi.deleted_at IS NULL",
        product_id
    ).fetch_all(&*state.db).await?;
    Ok(rows.into_iter().map(|r| serde_json::json!({"id":r.id,"url":r.url,"metadata":r.metadata,"created_at":r.created_at,"updated_at":r.updated_at})).collect())
}
