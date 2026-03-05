//! Admin product handlers
//! GET/POST /admin/products
//! GET/PUT/DELETE /admin/products/:id
//! GET/POST /admin/products/:id/variants
//! PUT/DELETE /admin/products/:id/variants/:variant_id
//! GET/POST/PUT/DELETE /admin/products/:id/options[/:option_id]

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, models::*, state::AppState};

// ─── List Products ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListProductsParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub q: Option<String>,
    pub status: Option<Vec<String>>,
    pub collection_id: Option<Vec<Uuid>>,
    pub category_id: Option<Vec<Uuid>>,
    pub id: Option<Vec<Uuid>>,
    pub handle: Option<String>,
    pub is_giftcard: Option<bool>,
    pub order: Option<String>,
    pub expand: Option<String>,
    pub fields: Option<String>,
}
fn default_limit() -> i64 { 50 }

#[derive(Debug, Serialize)]
pub struct ListProductsResponse {
    pub products: Vec<serde_json::Value>,
    pub count: i64,
    pub offset: i64,
    pub limit: i64,
}

pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<ListProductsParams>,
) -> Result<Json<ListProductsResponse>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, title, subtitle, description, handle, is_giftcard, status,
                  thumbnail, weight, length, height, width, hs_code, origin_country,
                  mid_code, material, collection_id, type_id, discountable,
                  external_id, metadata, created_at, updated_at, deleted_at
           FROM products
           WHERE deleted_at IS NULL
           ORDER BY created_at DESC
           LIMIT $1 OFFSET $2"#,
        params.limit,
        params.offset,
    )
    .fetch_all(&*state.db)
    .await?;

    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM products WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?
        .unwrap_or(0);

    let products: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "subtitle": r.subtitle,
                "description": r.description,
                "handle": r.handle,
                "is_giftcard": r.is_giftcard,
                "status": r.status,
                "thumbnail": r.thumbnail,
                "weight": r.weight,
                "length": r.length,
                "height": r.height,
                "width": r.width,
                "hs_code": r.hs_code,
                "origin_country": r.origin_country,
                "mid_code": r.mid_code,
                "material": r.material,
                "collection_id": r.collection_id,
                "type_id": r.type_id,
                "discountable": r.discountable,
                "external_id": r.external_id,
                "metadata": r.metadata,
                "created_at": r.created_at,
                "updated_at": r.updated_at,
                "deleted_at": r.deleted_at,
                "variants": [],
                "options": [],
                "images": [],
                "tags": [],
                "categories": [],
                "collection": null,
                "type": null,
            })
        })
        .collect();

    Ok(Json(ListProductsResponse {
        products,
        count,
        offset: params.offset,
        limit: params.limit,
    }))
}

// ─── Get Product ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product: serde_json::Value,
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, AppError> {
    let r = sqlx::query!(
        r#"SELECT id, title, subtitle, description, handle, is_giftcard, status,
                  thumbnail, weight, length, height, width, hs_code, origin_country,
                  mid_code, material, collection_id, type_id, discountable,
                  external_id, metadata, created_at, updated_at, deleted_at
           FROM products WHERE id = $1 AND deleted_at IS NULL"#,
        id
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    let variants = fetch_variants_for_product(&state, r.id).await?;
    let options = fetch_options_for_product(&state, r.id).await?;
    let images = fetch_images_for_product(&state, r.id).await?;

    let product = serde_json::json!({
        "id": r.id,
        "title": r.title,
        "subtitle": r.subtitle,
        "description": r.description,
        "handle": r.handle,
        "is_giftcard": r.is_giftcard,
        "status": r.status,
        "thumbnail": r.thumbnail,
        "weight": r.weight,
        "length": r.length,
        "height": r.height,
        "width": r.width,
        "hs_code": r.hs_code,
        "origin_country": r.origin_country,
        "mid_code": r.mid_code,
        "material": r.material,
        "collection_id": r.collection_id,
        "type_id": r.type_id,
        "discountable": r.discountable,
        "external_id": r.external_id,
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "deleted_at": r.deleted_at,
        "variants": variants,
        "options": options,
        "images": images,
        "tags": [],
        "categories": [],
        "collection": null,
        "type": null,
    });

    Ok(Json(ProductResponse { product }))
}

// ─── Create Product ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateProductPayload {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub is_giftcard: Option<bool>,
    pub status: Option<String>,
    pub thumbnail: Option<String>,
    pub weight: Option<f64>,
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub width: Option<f64>,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub collection_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub discountable: Option<bool>,
    pub external_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductPayload>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    let id = Uuid::new_v4();
    let handle = payload
        .handle
        .unwrap_or_else(|| crate::wizard::slugify(&payload.title));
    let status = payload.status.as_deref().unwrap_or("draft");

    let r = sqlx::query!(
        r#"INSERT INTO products
           (id, title, subtitle, description, handle, is_giftcard, status,
            thumbnail, weight, length, height, width, hs_code, origin_country,
            mid_code, material, collection_id, type_id, discountable,
            external_id, metadata, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,NOW(),NOW())
           RETURNING id, title, subtitle, description, handle, is_giftcard, status,
                     thumbnail, weight, length, height, width, hs_code, origin_country,
                     mid_code, material, collection_id, type_id, discountable,
                     external_id, metadata, created_at, updated_at, deleted_at"#,
        id,
        payload.title,
        payload.subtitle,
        payload.description,
        handle,
        payload.is_giftcard.unwrap_or(false),
        status,
        payload.thumbnail,
        payload.weight,
        payload.length,
        payload.height,
        payload.width,
        payload.hs_code,
        payload.origin_country,
        payload.mid_code,
        payload.material,
        payload.collection_id,
        payload.type_id,
        payload.discountable.unwrap_or(true),
        payload.external_id,
        payload.metadata,
    )
    .fetch_one(&*state.db)
    .await?;

    let product = serde_json::json!({
        "id": r.id,
        "title": r.title,
        "subtitle": r.subtitle,
        "description": r.description,
        "handle": r.handle,
        "is_giftcard": r.is_giftcard,
        "status": r.status,
        "thumbnail": r.thumbnail,
        "weight": r.weight,
        "length": r.length,
        "height": r.height,
        "width": r.width,
        "hs_code": r.hs_code,
        "origin_country": r.origin_country,
        "mid_code": r.mid_code,
        "material": r.material,
        "collection_id": r.collection_id,
        "type_id": r.type_id,
        "discountable": r.discountable,
        "external_id": r.external_id,
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "deleted_at": r.deleted_at,
        "variants": [],
        "options": [],
        "images": [],
        "tags": [],
        "categories": [],
        "collection": null,
        "type": null,
    });

    Ok((StatusCode::CREATED, Json(ProductResponse { product })))
}

// ─── Update Product ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateProductPayload {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub status: Option<String>,
    pub thumbnail: Option<String>,
    pub weight: Option<f64>,
    pub discountable: Option<bool>,
    pub collection_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<Json<ProductResponse>, AppError> {
    let r = sqlx::query!(
        r#"UPDATE products SET
           title       = COALESCE($2, title),
           subtitle    = COALESCE($3, subtitle),
           description = COALESCE($4, description),
           handle      = COALESCE($5, handle),
           status      = COALESCE($6, status),
           thumbnail   = COALESCE($7, thumbnail),
           weight      = COALESCE($8, weight),
           discountable = COALESCE($9, discountable),
           collection_id = COALESCE($10, collection_id),
           type_id     = COALESCE($11, type_id),
           metadata    = COALESCE($12, metadata),
           updated_at  = NOW()
           WHERE id = $1 AND deleted_at IS NULL
           RETURNING id, title, subtitle, description, handle, is_giftcard, status,
                     thumbnail, weight, length, height, width, hs_code, origin_country,
                     mid_code, material, collection_id, type_id, discountable,
                     external_id, metadata, created_at, updated_at, deleted_at"#,
        id,
        payload.title,
        payload.subtitle,
        payload.description,
        payload.handle,
        payload.status,
        payload.thumbnail,
        payload.weight,
        payload.discountable,
        payload.collection_id,
        payload.type_id,
        payload.metadata,
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    let variants = fetch_variants_for_product(&state, r.id).await?;
    let options = fetch_options_for_product(&state, r.id).await?;
    let images = fetch_images_for_product(&state, r.id).await?;

    let product = serde_json::json!({
        "id": r.id,
        "title": r.title,
        "subtitle": r.subtitle,
        "description": r.description,
        "handle": r.handle,
        "is_giftcard": r.is_giftcard,
        "status": r.status,
        "thumbnail": r.thumbnail,
        "weight": r.weight,
        "length": r.length,
        "height": r.height,
        "width": r.width,
        "hs_code": r.hs_code,
        "origin_country": r.origin_country,
        "mid_code": r.mid_code,
        "material": r.material,
        "collection_id": r.collection_id,
        "type_id": r.type_id,
        "discountable": r.discountable,
        "external_id": r.external_id,
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "deleted_at": r.deleted_at,
        "variants": variants,
        "options": options,
        "images": images,
        "tags": [],
        "categories": [],
        "collection": null,
        "type": null,
    });

    Ok(Json(ProductResponse { product }))
}

// ─── Delete Product ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub id: Uuid,
    pub object: &'static str,
    pub deleted: bool,
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, AppError> {
    sqlx::query!(
        "UPDATE products SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .execute(&*state.db)
    .await?;

    Ok(Json(DeleteResponse { id, object: "product", deleted: true }))
}

// ─── Variants ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VariantsResponse {
    pub variants: Vec<serde_json::Value>,
    pub count: i64,
    pub offset: i64,
    pub limit: i64,
}

pub async fn list_variants(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VariantsResponse>, AppError> {
    let variants = fetch_variants_for_product(&state, id).await?;
    let count = variants.len() as i64;
    Ok(Json(VariantsResponse { variants, count, offset: 0, limit: 100 }))
}

#[derive(Debug, Deserialize)]
pub struct CreateVariantPayload {
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
    pub inventory_quantity: Option<i32>,
    pub allow_backorder: Option<bool>,
    pub manage_inventory: Option<bool>,
    pub weight: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub prices: Option<Vec<CreatePricePayload>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePricePayload {
    pub currency_code: String,
    pub amount: i64,
    pub region_id: Option<Uuid>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct VariantResponse {
    pub variant: serde_json::Value,
}

pub async fn create_variant(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreateVariantPayload>,
) -> Result<(StatusCode, Json<VariantResponse>), AppError> {
    let id = Uuid::new_v4();
    let r = sqlx::query!(
        r#"INSERT INTO product_variants
           (id, product_id, title, sku, barcode, ean, upc, inventory_quantity,
            allow_backorder, manage_inventory, weight, metadata, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NOW(),NOW())
           RETURNING id, title, product_id, sku, barcode, ean, upc, inventory_quantity,
                     allow_backorder, manage_inventory, hs_code, origin_country,
                     mid_code, material, weight, length, height, width,
                     variant_rank, metadata, created_at, updated_at, deleted_at"#,
        id,
        product_id,
        payload.title,
        payload.sku,
        payload.barcode,
        payload.ean,
        payload.upc,
        payload.inventory_quantity.unwrap_or(0),
        payload.allow_backorder.unwrap_or(false),
        payload.manage_inventory.unwrap_or(true),
        payload.weight,
        payload.metadata,
    )
    .fetch_one(&*state.db)
    .await?;

    if let Some(prices) = payload.prices {
        for price in prices {
            let price_id = Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO money_amounts
                   (id, currency_code, amount, variant_id, region_id, min_quantity, max_quantity, created_at, updated_at)
                   VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW())"#,
                price_id,
                price.currency_code,
                price.amount,
                id,
                price.region_id,
                price.min_quantity,
                price.max_quantity,
            )
            .execute(&*state.db)
            .await?;
        }
    }

    let prices = fetch_prices_for_variant(&state, r.id).await?;
    let options = fetch_option_values_for_variant(&state, r.id).await?;

    let variant = serde_json::json!({
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
        "hs_code": r.hs_code,
        "origin_country": r.origin_country,
        "mid_code": r.mid_code,
        "material": r.material,
        "weight": r.weight,
        "length": r.length,
        "height": r.height,
        "width": r.width,
        "variant_rank": r.variant_rank,
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "deleted_at": r.deleted_at,
        "prices": prices,
        "options": options,
    });

    Ok((StatusCode::CREATED, Json(VariantResponse { variant })))
}

pub async fn update_variant(
    State(state): State<AppState>,
    Path((product_id, variant_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<VariantResponse>, AppError> {
    let r = sqlx::query!(
        r#"UPDATE product_variants SET
           title              = COALESCE($3::text, title),
           sku                = COALESCE($4::text, sku),
           inventory_quantity = COALESCE($5::int4, inventory_quantity),
           allow_backorder    = COALESCE($6::bool, allow_backorder),
           manage_inventory   = COALESCE($7::bool, manage_inventory),
           updated_at         = NOW()
           WHERE id = $1 AND product_id = $2 AND deleted_at IS NULL
           RETURNING id, title, product_id, sku, barcode, ean, upc, inventory_quantity,
                     allow_backorder, manage_inventory, hs_code, origin_country,
                     mid_code, material, weight, length, height, width,
                     variant_rank, metadata, created_at, updated_at, deleted_at"#,
        variant_id,
        product_id,
        payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("inventory_quantity").and_then(|v| v.as_i64()).map(|v| v as i32),
        payload.get("allow_backorder").and_then(|v| v.as_bool()),
        payload.get("manage_inventory").and_then(|v| v.as_bool()),
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Variant not found".into()))?;

    let prices = fetch_prices_for_variant(&state, r.id).await?;
    let options = fetch_option_values_for_variant(&state, r.id).await?;

    let variant = serde_json::json!({
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
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "deleted_at": r.deleted_at,
        "prices": prices,
        "options": options,
    });

    Ok(Json(VariantResponse { variant }))
}

pub async fn delete_variant(
    State(state): State<AppState>,
    Path((product_id, variant_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeleteResponse>, AppError> {
    sqlx::query!(
        "UPDATE product_variants SET deleted_at = NOW() WHERE id = $1 AND product_id = $2",
        variant_id,
        product_id
    )
    .execute(&*state.db)
    .await?;

    Ok(Json(DeleteResponse { id: variant_id, object: "product-variant", deleted: true }))
}

// ─── Options ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OptionsResponse {
    pub product: serde_json::Value,
}

pub async fn list_options(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let options = fetch_options_for_product(&state, product_id).await?;
    Ok(Json(serde_json::json!({ "product": { "id": product_id, "options": options } })))
}

pub async fn create_option(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("title is required".into()))?;

    sqlx::query!(
        "INSERT INTO product_options (id, title, product_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())",
        id, title, product_id
    )
    .execute(&*state.db)
    .await?;

    let options = fetch_options_for_product(&state, product_id).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "product": { "id": product_id, "options": options } }))))
}

pub async fn update_option(
    State(state): State<AppState>,
    Path((product_id, option_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let title = payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    sqlx::query!(
        "UPDATE product_options SET title = COALESCE($3, title), updated_at = NOW() WHERE id = $1 AND product_id = $2",
        option_id, product_id, title
    )
    .execute(&*state.db)
    .await?;

    let options = fetch_options_for_product(&state, product_id).await?;
    Ok(Json(serde_json::json!({ "product": { "id": product_id, "options": options } })))
}

pub async fn delete_option(
    State(state): State<AppState>,
    Path((product_id, option_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeleteResponse>, AppError> {
    sqlx::query!(
        "DELETE FROM product_options WHERE id = $1 AND product_id = $2",
        option_id, product_id
    )
    .execute(&*state.db)
    .await?;

    Ok(Json(DeleteResponse { id: option_id, object: "option", deleted: true }))
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

async fn fetch_variants_for_product(
    state: &AppState,
    product_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, title, product_id, sku, barcode, ean, upc, inventory_quantity,
                  allow_backorder, manage_inventory, hs_code, origin_country,
                  mid_code, material, weight, length, height, width,
                  variant_rank, metadata, created_at, updated_at, deleted_at
           FROM product_variants
           WHERE product_id = $1 AND deleted_at IS NULL
           ORDER BY variant_rank ASC NULLS LAST"#,
        product_id
    )
    .fetch_all(&*state.db)
    .await?;

    let mut variants = Vec::new();
    for r in rows {
        let prices = fetch_prices_for_variant(state, r.id).await?;
        let options = fetch_option_values_for_variant(state, r.id).await?;
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
            "hs_code": r.hs_code,
            "origin_country": r.origin_country,
            "mid_code": r.mid_code,
            "material": r.material,
            "weight": r.weight,
            "length": r.length,
            "height": r.height,
            "width": r.width,
            "variant_rank": r.variant_rank,
            "metadata": r.metadata,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
            "deleted_at": r.deleted_at,
            "prices": prices,
            "options": options,
        }));
    }
    Ok(variants)
}

async fn fetch_prices_for_variant(
    state: &AppState,
    variant_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, currency_code, amount, variant_id, region_id, price_list_id,
                  min_quantity, max_quantity, created_at, updated_at
           FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL"#,
        variant_id
    )
    .fetch_all(&*state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::json!({
            "id": r.id,
            "currency_code": r.currency_code,
            "amount": r.amount,
            "variant_id": r.variant_id,
            "region_id": r.region_id,
            "price_list_id": r.price_list_id,
            "min_quantity": r.min_quantity,
            "max_quantity": r.max_quantity,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
        }))
        .collect())
}

async fn fetch_options_for_product(
    state: &AppState,
    product_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT id, title, product_id, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL",
        product_id
    )
    .fetch_all(&*state.db)
    .await?;

    let mut opts = Vec::new();
    for r in rows {
        let values = sqlx::query!(
            "SELECT id, value, option_id, variant_id, metadata, created_at, updated_at FROM product_option_values WHERE option_id = $1",
            r.id
        )
        .fetch_all(&*state.db)
        .await?
        .into_iter()
        .map(|v| serde_json::json!({
            "id": v.id,
            "value": v.value,
            "option_id": v.option_id,
            "variant_id": v.variant_id,
            "metadata": v.metadata,
            "created_at": v.created_at,
            "updated_at": v.updated_at,
        }))
        .collect::<Vec<_>>();

        opts.push(serde_json::json!({
            "id": r.id,
            "title": r.title,
            "product_id": r.product_id,
            "metadata": r.metadata,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
            "values": values,
        }));
    }
    Ok(opts)
}

async fn fetch_option_values_for_variant(
    state: &AppState,
    variant_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT id, value, option_id, variant_id, metadata, created_at, updated_at FROM product_option_values WHERE variant_id = $1",
        variant_id
    )
    .fetch_all(&*state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::json!({
            "id": r.id,
            "value": r.value,
            "option_id": r.option_id,
            "variant_id": r.variant_id,
            "metadata": r.metadata,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
        }))
        .collect())
}

async fn fetch_images_for_product(
    state: &AppState,
    product_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT pi.id, pi.url, pi.metadata, pi.created_at, pi.updated_at
           FROM product_images pi
           JOIN product_images_products pip ON pip.image_id = pi.id
           WHERE pip.product_id = $1 AND pi.deleted_at IS NULL"#,
        product_id
    )
    .fetch_all(&*state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::json!({
            "id": r.id,
            "url": r.url,
            "metadata": r.metadata,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
        }))
        .collect())
}
