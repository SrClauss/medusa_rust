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
    let variants = sqlx::query("SELECT id, title, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, unit_price, variant_rank, hs_code, origin_country, mid_code, material, weight, length, height, width, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST")
        .bind(id).fetch_all(&*state.db).await?;
    let mut variant_jsons = Vec::new();
    for v in &variants {
        let vid: Uuid = v.get("id");
        let prices = sqlx::query("SELECT id, currency_code, amount, variant_id, region_id, created_at, updated_at FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL AND price_list_id IS NULL")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|p| serde_json::json!({"id":p.get::<Uuid,_>("id"),"currency_code":p.get::<String,_>("currency_code"),"amount":p.get::<i64,_>("amount"),"variant_id":p.get::<Uuid,_>("variant_id"),"region_id":p.get::<Option<Uuid>,_>("region_id"),"created_at":p.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":p.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
        let options = sqlx::query("SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE variant_id = $1")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"value":o.get::<String,_>("value"),"option_id":o.get::<Uuid,_>("option_id"),"variant_id":o.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
        variant_jsons.push(serde_json::json!({"id":vid,"product_id":id,"title":v.get::<String,_>("title"),"sku":v.get::<Option<String>,_>("sku"),"barcode":v.get::<Option<String>,_>("barcode"),"ean":v.get::<Option<String>,_>("ean"),"upc":v.get::<Option<String>,_>("upc"),"inventory_quantity":v.get::<i32,_>("inventory_quantity"),"allow_backorder":v.get::<bool,_>("allow_backorder"),"manage_inventory":v.get::<bool,_>("manage_inventory"),"hs_code":v.get::<Option<String>,_>("hs_code"),"origin_country":v.get::<Option<String>,_>("origin_country"),"mid_code":v.get::<Option<String>,_>("mid_code"),"material":v.get::<Option<String>,_>("material"),"weight":v.get::<Option<f64>,_>("weight"),"length":v.get::<Option<f64>,_>("length"),"height":v.get::<Option<f64>,_>("height"),"width":v.get::<Option<f64>,_>("width"),"variant_rank":v.get::<Option<i32>,_>("variant_rank"),"metadata":v.get::<Option<serde_json::Value>,_>("metadata"),"created_at":v.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":v.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"deleted_at":null,"prices":prices,"options":options}));
    }
    let options = sqlx::query("SELECT id, title, metadata, created_at, updated_at FROM product_options WHERE product_id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_all(&*state.db).await?;
    let mut option_jsons = Vec::new();
    for o in &options {
        let oid: Uuid = o.get("id");
        let values = sqlx::query("SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE option_id = $1")
            .bind(oid).fetch_all(&*state.db).await?
            .into_iter().map(|v| serde_json::json!({"id":v.get::<Uuid,_>("id"),"value":v.get::<String,_>("value"),"option_id":v.get::<Uuid,_>("option_id"),"variant_id":v.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
        option_jsons.push(serde_json::json!({"id":oid,"title":o.get::<String,_>("title"),"product_id":id,"metadata":o.get::<Option<serde_json::Value>,_>("metadata"),"created_at":o.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":o.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"values":values}));
    }
    let images = sqlx::query("SELECT pi.id, pi.url, pi.metadata, pi.created_at, pi.updated_at FROM product_images pi JOIN product_images_products pip ON pip.image_id = pi.id WHERE pip.product_id = $1 AND pi.deleted_at IS NULL")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|i| serde_json::json!({"id":i.get::<Uuid,_>("id"),"url":i.get::<String,_>("url"),"metadata":i.get::<Option<serde_json::Value>,_>("metadata"),"created_at":i.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":i.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
    let tags = sqlx::query("SELECT pt.id, pt.value FROM product_tags pt JOIN product_tags_products ptp ON ptp.tag_id = pt.id WHERE ptp.product_id = $1 AND pt.deleted_at IS NULL")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|t| serde_json::json!({"id":t.get::<Uuid,_>("id"),"value":t.get::<String,_>("value")})).collect::<Vec<_>>();
    let categories = sqlx::query("SELECT pc.id, pc.name, pc.handle FROM product_categories pc JOIN product_category_products pcp ON pcp.category_id = pc.id WHERE pcp.product_id = $1")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|c| serde_json::json!({"id":c.get::<Uuid,_>("id"),"name":c.get::<String,_>("name"),"handle":c.get::<String,_>("handle")})).collect::<Vec<_>>();
    let type_id: Option<Uuid> = r.get("type_id");
    let product_type = if let Some(tid) = type_id {
        sqlx::query("SELECT id, value FROM product_types WHERE id = $1")
            .bind(tid).fetch_optional(&*state.db).await?
            .map(|t| serde_json::json!({"id":t.get::<Uuid,_>("id"),"value":t.get::<String,_>("value")}))
    } else { None };
    Ok(serde_json::json!({
        "id":r.get::<Uuid,_>("id"),
        "title":r.get::<String,_>("title"),
        "subtitle":r.get::<Option<String>,_>("subtitle"),
        "description":r.get::<Option<String>,_>("description"),
        "handle":r.get::<String,_>("handle"),
        "is_giftcard":r.get::<bool,_>("is_giftcard"),
        "status":r.get::<String,_>("status"),
        "thumbnail":r.get::<Option<String>,_>("thumbnail"),
        "weight":r.get::<Option<f64>,_>("weight"),
        "length":r.get::<Option<f64>,_>("length"),
        "height":r.get::<Option<f64>,_>("height"),
        "width":r.get::<Option<f64>,_>("width"),
        "hs_code":r.get::<Option<String>,_>("hs_code"),
        "origin_country":r.get::<Option<String>,_>("origin_country"),
        "mid_code":r.get::<Option<String>,_>("mid_code"),
        "material":r.get::<Option<String>,_>("material"),
        "collection_id":r.get::<Option<Uuid>,_>("collection_id"),
        "type_id":type_id,
        "type":product_type,
        "discountable":r.get::<bool,_>("discountable"),
        "external_id":r.get::<Option<String>,_>("external_id"),
        "metadata":r.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
        "deleted_at":null,
        "variants":variant_jsons,
        "options":option_jsons,
        "images":images,
        "tags":tags,
        "categories":categories,
        "collection":null,
        "profile_id":null,
    }))
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, type_id, discountable, weight, length, height, width, hs_code, origin_country, mid_code, material, external_id, metadata, created_at, updated_at FROM products WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE deleted_at IS NULL").fetch_one(&*state.db).await?;
    let mut products = Vec::new();
    for r in &rows { products.push(build_product(&state, r.get("id"), r).await?); }
    Ok(Json(serde_json::json!({"products":products,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, type_id, discountable, weight, length, height, width, hs_code, origin_country, mid_code, material, external_id, metadata, created_at, updated_at FROM products WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Product not found".into()))?;
    Ok(Json(serde_json::json!({"product":build_product(&state, id, &r).await?})))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let handle = payload.get("handle").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| crate::wizard::slugify(title));
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("draft");
    // Resolve or create product type
    let type_id: Option<Uuid> = if let Some(t) = payload.get("type").and_then(|v| v.as_object()) {
        if let Some(val) = t.get("value").and_then(|v| v.as_str()) {
            let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM product_types WHERE value = $1 AND deleted_at IS NULL LIMIT 1").bind(val).fetch_optional(&*state.db).await?;
            if let Some(eid) = existing { Some(eid) } else {
                let tid = Uuid::new_v4();
                sqlx::query("INSERT INTO product_types (id, value, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(tid).bind(val).execute(&*state.db).await?;
                Some(tid)
            }
        } else { None }
    } else { None };
    let r = sqlx::query("INSERT INTO products (id, title, subtitle, description, handle, is_giftcard, status, thumbnail, type_id, discountable, weight, length, height, width, hs_code, origin_country, mid_code, material, external_id, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,NOW(),NOW()) RETURNING id, title, subtitle, description, handle, is_giftcard, status, thumbnail, collection_id, type_id, discountable, weight, length, height, width, hs_code, origin_country, mid_code, material, external_id, metadata, created_at, updated_at")
        .bind(id).bind(title)
        .bind(payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(handle).bind(payload.get("is_giftcard").and_then(|v| v.as_bool()).unwrap_or(false)).bind(status)
        .bind(payload.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(type_id)
        .bind(payload.get("discountable").and_then(|v| v.as_bool()).unwrap_or(true))
        .bind(payload.get("weight").and_then(|v| v.as_f64()))
        .bind(payload.get("length").and_then(|v| v.as_f64()))
        .bind(payload.get("height").and_then(|v| v.as_f64()))
        .bind(payload.get("width").and_then(|v| v.as_f64()))
        .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("origin_country").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("external_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("metadata").cloned())
        .fetch_one(&*state.db).await?;
    // Handle images (array of URL strings)
    if let Some(images) = payload.get("images").and_then(|v| v.as_array()) {
        for img in images {
            if let Some(url) = img.as_str() {
                let iid = Uuid::new_v4();
                sqlx::query("INSERT INTO product_images (id, url, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(iid).bind(url).execute(&*state.db).await?;
                sqlx::query("INSERT INTO product_images_products (product_id, image_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(id).bind(iid).execute(&*state.db).await?;
            }
        }
    }
    // Handle tags (array of { value: string })
    if let Some(tags) = payload.get("tags").and_then(|v| v.as_array()) {
        for tag in tags {
            if let Some(val) = tag.get("value").and_then(|v| v.as_str()) {
                let existing_tag: Option<Uuid> = sqlx::query_scalar("SELECT id FROM product_tags WHERE value = $1 AND deleted_at IS NULL LIMIT 1").bind(val).fetch_optional(&*state.db).await?;
                let tag_id = if let Some(eid) = existing_tag { eid } else {
                    let tid = Uuid::new_v4();
                    sqlx::query("INSERT INTO product_tags (id, value, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(tid).bind(val).execute(&*state.db).await?;
                    tid
                };
                sqlx::query("INSERT INTO product_tags_products (product_id, tag_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(id).bind(tag_id).execute(&*state.db).await?;
            }
        }
    }
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
            let allow_backorder = var.get("allow_backorder").and_then(|v| v.as_bool()).unwrap_or(false);
            let manage_inventory = var.get("manage_inventory").and_then(|v| v.as_bool()).unwrap_or(true);
            sqlx::query("INSERT INTO product_variants (id, product_id, title, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, hs_code, origin_country, mid_code, material, weight, length, height, width, variant_rank, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,NOW(),NOW())")
                .bind(vid).bind(id).bind(var_title)
                .bind(var.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("barcode").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("ean").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("upc").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(inv).bind(allow_backorder).bind(manage_inventory)
                .bind(var.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("origin_country").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .bind(var.get("weight").and_then(|v| v.as_f64()))
                .bind(var.get("length").and_then(|v| v.as_f64()))
                .bind(var.get("height").and_then(|v| v.as_f64()))
                .bind(var.get("width").and_then(|v| v.as_f64()))
                .bind(rank as i32)
                .bind(var.get("metadata").cloned())
                .execute(&*state.db).await?;
            if let Some(prices) = var.get("prices").and_then(|v| v.as_array()) {
                for price in prices {
                    let pid = Uuid::new_v4();
                    let amount: i64 = price.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cc = price.get("currency_code").and_then(|v| v.as_str()).unwrap_or("usd");
                    sqlx::query("INSERT INTO money_amounts (id, currency_code, amount, variant_id, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW())").bind(pid).bind(cc).bind(amount).bind(vid).execute(&*state.db).await?;
                }
            }
            // Handle option values
            if let Some(opt_vals) = var.get("options").and_then(|v| v.as_array()) {
                for opt_val in opt_vals {
                    if let Some(val) = opt_val.get("value").and_then(|v| v.as_str()) {
                        // Find option by title if option_id not provided
                        let opt_id: Option<Uuid> = if let Some(oid) = opt_val.get("option_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) {
                            Some(oid)
                        } else {
                            sqlx::query_scalar("SELECT id FROM product_options WHERE product_id = $1 LIMIT 1").bind(id).fetch_optional(&*state.db).await?
                        };
                        if let Some(oid) = opt_id {
                            let ovid = Uuid::new_v4();
                            sqlx::query("INSERT INTO product_option_values (id, value, option_id, variant_id, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW()) ON CONFLICT DO NOTHING").bind(ovid).bind(val).bind(oid).bind(vid).execute(&*state.db).await?;
                        }
                    }
                }
            }
        }
    }
    Ok((StatusCode::CREATED, Json(serde_json::json!({"product":build_product(&state, id, &r).await?}))))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    // Resolve or create product type if provided
    let type_id: Option<Option<Uuid>> = if let Some(t) = payload.get("type") {
        if t.is_null() {
            Some(None)
        } else if let Some(obj) = t.as_object() {
            if let Some(val) = obj.get("value").and_then(|v| v.as_str()) {
                let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM product_types WHERE value = $1 AND deleted_at IS NULL LIMIT 1").bind(val).fetch_optional(&*state.db).await?;
                let tid = if let Some(eid) = existing { eid } else {
                    let tid = Uuid::new_v4();
                    sqlx::query("INSERT INTO product_types (id, value, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(tid).bind(val).execute(&*state.db).await?;
                    tid
                };
                Some(Some(tid))
            } else { None }
        } else { None }
    } else { None };
    sqlx::query("UPDATE products SET title=COALESCE($2,title), subtitle=COALESCE($3,subtitle), description=COALESCE($4,description), status=COALESCE($5,status), thumbnail=COALESCE($6,thumbnail), weight=COALESCE($7,weight), length=COALESCE($8,length), height=COALESCE($9,height), width=COALESCE($10,width), hs_code=COALESCE($11,hs_code), origin_country=COALESCE($12,origin_country), mid_code=COALESCE($13,mid_code), material=COALESCE($14,material), discountable=COALESCE($15,discountable), metadata=COALESCE($16,metadata), updated_at=NOW() WHERE id=$1 AND deleted_at IS NULL")
        .bind(id)
        .bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("weight").and_then(|v| v.as_f64()))
        .bind(payload.get("length").and_then(|v| v.as_f64()))
        .bind(payload.get("height").and_then(|v| v.as_f64()))
        .bind(payload.get("width").and_then(|v| v.as_f64()))
        .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("origin_country").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("discountable").and_then(|v| v.as_bool()))
        .bind(payload.get("metadata").cloned())
        .execute(&*state.db).await?;
    // Update type_id if type was provided
    if let Some(tid) = type_id {
        sqlx::query("UPDATE products SET type_id = $2, updated_at = NOW() WHERE id = $1").bind(id).bind(tid).execute(&*state.db).await?;
    }
    // Update images if provided
    if let Some(images) = payload.get("images").and_then(|v| v.as_array()) {
        // Remove existing images
        sqlx::query("DELETE FROM product_images_products WHERE product_id = $1").bind(id).execute(&*state.db).await?;
        for img in images {
            if let Some(url) = img.as_str() {
                let iid = Uuid::new_v4();
                sqlx::query("INSERT INTO product_images (id, url, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(iid).bind(url).execute(&*state.db).await?;
                sqlx::query("INSERT INTO product_images_products (product_id, image_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(id).bind(iid).execute(&*state.db).await?;
            }
        }
    }
    // Update tags if provided
    if let Some(tags) = payload.get("tags").and_then(|v| v.as_array()) {
        sqlx::query("DELETE FROM product_tags_products WHERE product_id = $1").bind(id).execute(&*state.db).await?;
        for tag in tags {
            if let Some(val) = tag.get("value").and_then(|v| v.as_str()) {
                let existing_tag: Option<Uuid> = sqlx::query_scalar("SELECT id FROM product_tags WHERE value = $1 AND deleted_at IS NULL LIMIT 1").bind(val).fetch_optional(&*state.db).await?;
                let tag_id = if let Some(eid) = existing_tag { eid } else {
                    let tid = Uuid::new_v4();
                    sqlx::query("INSERT INTO product_tags (id, value, created_at, updated_at) VALUES ($1,$2,NOW(),NOW())").bind(tid).bind(val).execute(&*state.db).await?;
                    tid
                };
                sqlx::query("INSERT INTO product_tags_products (product_id, tag_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(id).bind(tag_id).execute(&*state.db).await?;
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE products SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":id,"object":"product","deleted":true})))
}

// ─── Variants ─────────────────────────────────────────────────────────────────
pub async fn list_variants(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, variant_rank, hs_code, origin_country, mid_code, material, weight, length, height, width, metadata, created_at, updated_at FROM product_variants WHERE product_id = $1 AND deleted_at IS NULL ORDER BY variant_rank ASC NULLS LAST").bind(id).fetch_all(&*state.db).await?;
    let mut variants = Vec::new();
    for v in &rows {
        let vid: Uuid = v.get("id");
        let prices = sqlx::query("SELECT id, currency_code, amount, variant_id, region_id, created_at, updated_at FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL AND price_list_id IS NULL")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|p| serde_json::json!({"id":p.get::<Uuid,_>("id"),"currency_code":p.get::<String,_>("currency_code"),"amount":p.get::<i64,_>("amount"),"variant_id":p.get::<Uuid,_>("variant_id"),"region_id":p.get::<Option<Uuid>,_>("region_id")})).collect::<Vec<_>>();
        let options = sqlx::query("SELECT id, value, option_id, variant_id FROM product_option_values WHERE variant_id = $1")
            .bind(vid).fetch_all(&*state.db).await?
            .into_iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"value":o.get::<String,_>("value"),"option_id":o.get::<Uuid,_>("option_id"),"variant_id":o.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
        variants.push(serde_json::json!({"id":vid,"title":v.get::<String,_>("title"),"product_id":id,"sku":v.get::<Option<String>,_>("sku"),"barcode":v.get::<Option<String>,_>("barcode"),"ean":v.get::<Option<String>,_>("ean"),"upc":v.get::<Option<String>,_>("upc"),"inventory_quantity":v.get::<i32,_>("inventory_quantity"),"allow_backorder":v.get::<bool,_>("allow_backorder"),"manage_inventory":v.get::<bool,_>("manage_inventory"),"variant_rank":v.get::<Option<i32>,_>("variant_rank"),"metadata":v.get::<Option<serde_json::Value>,_>("metadata"),"created_at":v.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":v.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"prices":prices,"options":options}));
    }
    let count = variants.len() as i64;
    Ok(Json(serde_json::json!({"variants":variants,"count":count})))
}

pub async fn create_variant(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let vid = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("title required".into()))?;
    let inv: i32 = payload.get("inventory_quantity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let allow_backorder = payload.get("allow_backorder").and_then(|v| v.as_bool()).unwrap_or(false);
    let manage_inventory = payload.get("manage_inventory").and_then(|v| v.as_bool()).unwrap_or(true);
    sqlx::query("INSERT INTO product_variants (id, product_id, title, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, hs_code, origin_country, mid_code, material, weight, length, height, width, metadata, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,NOW(),NOW())")
        .bind(vid).bind(id).bind(title)
        .bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("barcode").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("ean").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("upc").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(inv).bind(allow_backorder).bind(manage_inventory)
        .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("origin_country").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("weight").and_then(|v| v.as_f64()))
        .bind(payload.get("length").and_then(|v| v.as_f64()))
        .bind(payload.get("height").and_then(|v| v.as_f64()))
        .bind(payload.get("width").and_then(|v| v.as_f64()))
        .bind(payload.get("metadata").cloned())
        .execute(&*state.db).await?;
    if let Some(prices) = payload.get("prices").and_then(|v| v.as_array()) {
        for price in prices {
            let pid = Uuid::new_v4();
            let amount: i64 = price.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
            let cc = price.get("currency_code").and_then(|v| v.as_str()).unwrap_or("usd");
            sqlx::query("INSERT INTO money_amounts (id, currency_code, amount, variant_id, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW())").bind(pid).bind(cc).bind(amount).bind(vid).execute(&*state.db).await?;
        }
    }
    get_variant(axum::extract::State(state), axum::extract::Path((id, vid))).await.map(|r| (StatusCode::CREATED, r))
}

pub async fn update_variant(State(state): State<AppState>, Path((pid, vid)): Path<(Uuid,Uuid)>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let inv: Option<i32> = payload.get("inventory_quantity").and_then(|v| v.as_i64()).map(|v| v as i32);
    sqlx::query("UPDATE product_variants SET title=COALESCE($3,title), sku=COALESCE($4,sku), barcode=COALESCE($5,barcode), ean=COALESCE($6,ean), upc=COALESCE($7,upc), inventory_quantity=COALESCE($8,inventory_quantity), allow_backorder=COALESCE($9,allow_backorder), manage_inventory=COALESCE($10,manage_inventory), hs_code=COALESCE($11,hs_code), origin_country=COALESCE($12,origin_country), mid_code=COALESCE($13,mid_code), material=COALESCE($14,material), weight=COALESCE($15,weight), length=COALESCE($16,length), height=COALESCE($17,height), width=COALESCE($18,width), metadata=COALESCE($19,metadata), updated_at=NOW() WHERE id=$1 AND product_id=$2")
        .bind(vid).bind(pid)
        .bind(payload.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("barcode").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("ean").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("upc").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(inv)
        .bind(payload.get("allow_backorder").and_then(|v| v.as_bool()))
        .bind(payload.get("manage_inventory").and_then(|v| v.as_bool()))
        .bind(payload.get("hs_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("origin_country").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("mid_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("material").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("weight").and_then(|v| v.as_f64()))
        .bind(payload.get("length").and_then(|v| v.as_f64()))
        .bind(payload.get("height").and_then(|v| v.as_f64()))
        .bind(payload.get("width").and_then(|v| v.as_f64()))
        .bind(payload.get("metadata").cloned())
        .execute(&*state.db).await?;
    // Update prices if provided
    if let Some(prices) = payload.get("prices").and_then(|v| v.as_array()) {
        sqlx::query("DELETE FROM money_amounts WHERE variant_id = $1 AND price_list_id IS NULL").bind(vid).execute(&*state.db).await?;
        for price in prices {
            let pid2 = Uuid::new_v4();
            let amount: i64 = price.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
            let cc = price.get("currency_code").and_then(|v| v.as_str()).unwrap_or("usd");
            sqlx::query("INSERT INTO money_amounts (id, currency_code, amount, variant_id, created_at, updated_at) VALUES ($1,$2,$3,$4,NOW(),NOW())").bind(pid2).bind(cc).bind(amount).bind(vid).execute(&*state.db).await?;
        }
    }
    get_variant(axum::extract::State(state), axum::extract::Path((pid, vid))).await
}

pub async fn delete_variant(State(state): State<AppState>, Path((_pid, vid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE product_variants SET deleted_at = NOW() WHERE id = $1").bind(vid).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"id":vid,"object":"product-variant","deleted":true})))
}

pub async fn get_variant(State(state): State<AppState>, Path((_pid, vid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, sku, barcode, ean, upc, inventory_quantity, allow_backorder, manage_inventory, product_id, variant_rank, hs_code, origin_country, mid_code, material, weight, length, height, width, metadata, created_at, updated_at FROM product_variants WHERE id = $1 AND deleted_at IS NULL").bind(vid).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Variant not found".into()))?;
    let pid: Uuid = r.get("product_id");
    let prices = sqlx::query("SELECT id, currency_code, amount, variant_id, region_id, created_at, updated_at FROM money_amounts WHERE variant_id = $1 AND deleted_at IS NULL AND price_list_id IS NULL")
        .bind(vid).fetch_all(&*state.db).await?
        .into_iter().map(|p| serde_json::json!({"id":p.get::<Uuid,_>("id"),"currency_code":p.get::<String,_>("currency_code"),"amount":p.get::<i64,_>("amount"),"variant_id":p.get::<Uuid,_>("variant_id"),"region_id":p.get::<Option<Uuid>,_>("region_id"),"created_at":p.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":p.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
    let options = sqlx::query("SELECT id, value, option_id, variant_id, created_at, updated_at FROM product_option_values WHERE variant_id = $1")
        .bind(vid).fetch_all(&*state.db).await?
        .into_iter().map(|o| serde_json::json!({"id":o.get::<Uuid,_>("id"),"value":o.get::<String,_>("value"),"option_id":o.get::<Uuid,_>("option_id"),"variant_id":o.get::<Uuid,_>("variant_id")})).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({"variant":{
        "id":r.get::<Uuid,_>("id"),
        "title":r.get::<String,_>("title"),
        "product_id":pid,
        "sku":r.get::<Option<String>,_>("sku"),
        "barcode":r.get::<Option<String>,_>("barcode"),
        "ean":r.get::<Option<String>,_>("ean"),
        "upc":r.get::<Option<String>,_>("upc"),
        "inventory_quantity":r.get::<i32,_>("inventory_quantity"),
        "allow_backorder":r.get::<bool,_>("allow_backorder"),
        "manage_inventory":r.get::<bool,_>("manage_inventory"),
        "hs_code":r.get::<Option<String>,_>("hs_code"),
        "origin_country":r.get::<Option<String>,_>("origin_country"),
        "mid_code":r.get::<Option<String>,_>("mid_code"),
        "material":r.get::<Option<String>,_>("material"),
        "weight":r.get::<Option<f64>,_>("weight"),
        "length":r.get::<Option<f64>,_>("length"),
        "height":r.get::<Option<f64>,_>("height"),
        "width":r.get::<Option<f64>,_>("width"),
        "variant_rank":r.get::<Option<i32>,_>("variant_rank"),
        "metadata":r.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
        "deleted_at":null,
        "prices":prices,
        "options":options,
    }})))
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
