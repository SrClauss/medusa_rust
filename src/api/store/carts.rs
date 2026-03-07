//! Store cart handlers — full checkout flow
use axum::{extract::{Path, State}, http::StatusCode, Json};
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

// ─── Cart helpers ─────────────────────────────────────────────────────────────

async fn fetch_cart(state: &AppState, cart_id: Uuid) -> Result<serde_json::Value, AppError> {
    let c = sqlx::query("SELECT id, email, region_id, customer_id, billing_address_id, shipping_address_id, cart_type, completed_at, payment_authorized_at, idempotency_key, context, sales_channel_id, metadata, created_at, updated_at FROM carts WHERE id = $1 AND deleted_at IS NULL")
        .bind(cart_id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Cart not found".into()))?;
    let region_id: Uuid = c.get("region_id");
    let region = sqlx::query("SELECT id, name, currency_code, tax_rate FROM regions WHERE id = $1").bind(region_id).fetch_optional(&*state.db).await?.map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"name":r.get::<String,_>("name"),"currency_code":r.get::<String,_>("currency_code"),"tax_rate":r.get::<f64,_>("tax_rate")}));
    let items = sqlx::query("SELECT li.id, li.cart_id, li.title, li.description, li.thumbnail, li.is_giftcard, li.should_merge, li.allow_discounts, li.quantity, li.unit_price, li.variant_id, li.created_at, li.updated_at FROM line_items li WHERE li.cart_id = $1 AND li.order_id IS NULL AND li.claim_order_id IS NULL AND li.swap_id IS NULL")
        .bind(cart_id).fetch_all(&*state.db).await?
        .into_iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"cart_id":r.get::<Uuid,_>("cart_id"),"title":r.get::<String,_>("title"),"description":r.get::<Option<String>,_>("description"),"thumbnail":r.get::<Option<String>,_>("thumbnail"),"is_giftcard":r.get::<bool,_>("is_giftcard"),"should_merge":r.get::<bool,_>("should_merge"),"allow_discounts":r.get::<bool,_>("allow_discounts"),"quantity":r.get::<i32,_>("quantity"),"unit_price":r.get::<i64,_>("unit_price"),"variant_id":r.get::<Option<Uuid>,_>("variant_id"),"subtotal":r.get::<i64,_>("unit_price")*r.get::<i32,_>("quantity") as i64,"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"variant":null,"adjustments":[],"tax_lines":[]})).collect::<Vec<_>>();
    let shipping_methods = sqlx::query("SELECT id, cart_id, shipping_option_id, price, data, created_at, updated_at FROM shipping_methods WHERE cart_id = $1")
        .bind(cart_id).fetch_all(&*state.db).await?
        .into_iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"cart_id":r.get::<Uuid,_>("cart_id"),"shipping_option_id":r.get::<Uuid,_>("shipping_option_id"),"price":r.get::<i64,_>("price"),"data":r.get::<Option<serde_json::Value>,_>("data"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at")})).collect::<Vec<_>>();
    let payment_sessions = sqlx::query("SELECT id, cart_id, provider_id, is_selected, is_initiated, status, data, amount, created_at, updated_at FROM payment_sessions WHERE cart_id = $1")
        .bind(cart_id).fetch_all(&*state.db).await?
        .into_iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"cart_id":r.get::<Uuid,_>("cart_id"),"provider_id":r.get::<String,_>("provider_id"),"is_selected":r.get::<bool,_>("is_selected"),"is_initiated":r.get::<bool,_>("is_initiated"),"status":r.get::<String,_>("status"),"data":r.get::<Option<serde_json::Value>,_>("data"),"amount":r.get::<i64,_>("amount"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")})).collect::<Vec<_>>();
    let selected_payment_session = payment_sessions.iter().find(|s| s["is_selected"].as_bool().unwrap_or(false)).cloned();
    // Resolve billing and shipping addresses
    let billing_address = if let Some(aid) = c.get::<Option<Uuid>,_>("billing_address_id") {
        sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code FROM addresses WHERE id = $1")
            .bind(aid).fetch_optional(&*state.db).await?
            .map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")}))
    } else { None };
    let shipping_address = if let Some(aid) = c.get::<Option<Uuid>,_>("shipping_address_id") {
        sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code FROM addresses WHERE id = $1")
            .bind(aid).fetch_optional(&*state.db).await?
            .map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")}))
    } else { None };
    let subtotal: i64 = items.iter().map(|i| i["subtotal"].as_i64().unwrap_or(0)).sum();
    let shipping_total: i64 = shipping_methods.iter().map(|s| s["price"].as_i64().unwrap_or(0)).sum();
    let tax_rate: f64 = region.as_ref().and_then(|r| r["tax_rate"].as_f64()).unwrap_or(0.0);
    let tax_total: i64 = ((subtotal as f64) * tax_rate / 100.0).round() as i64;
    let total = subtotal + shipping_total + tax_total;
    // fetch applied discounts
    let discounts = sqlx::query("SELECT d.id, d.code, d.is_disabled, r.type AS rule_type, r.value AS rule_value FROM discounts d JOIN cart_discounts cd ON d.id = cd.discount_id JOIN discount_rules r ON d.rule_id = r.id WHERE cd.cart_id = $1")
        .bind(cart_id).fetch_all(&*state.db).await?
        .into_iter().map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid,_>("id"),
                "code": r.get::<String,_>("code"),
                "is_disabled": r.get::<bool,_>("is_disabled"),
                // rule info for client-side debugging
                "rule": {
                    "type": r.get::<String,_>("rule_type"),
                    "value": r.get::<i64,_>("rule_value"),
                }
            })
        }).collect::<Vec<_>>();
    // compute discount_total using simple percentage/fixed rules
    let mut discount_total: i64 = 0;
    for d in &discounts {
        if let Some(rule) = d.get("rule") {
            if let Some(rtype) = rule.get("type").and_then(|v| v.as_str()) {
                let rval = rule.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                if rtype == "percentage" {
                    discount_total += ((subtotal as f64) * (rval as f64) / 100.0).round() as i64;
                } else {
                    discount_total += rval;
                }
            }
        }
    }
    // ensure totals don't go negative
    if discount_total > total { discount_total = total; }
    let total = subtotal + shipping_total + tax_total - discount_total;
    Ok(serde_json::json!({
        "id":c.get::<Uuid,_>("id"),
        "email":c.get::<Option<String>,_>("email"),
        "region_id":region_id,"region":region,
        "customer_id":c.get::<Option<Uuid>,_>("customer_id"),
        "billing_address_id":c.get::<Option<Uuid>,_>("billing_address_id"),
        "billing_address":billing_address,
        "shipping_address_id":c.get::<Option<Uuid>,_>("shipping_address_id"),
        "shipping_address":shipping_address,
        "type":c.get::<String,_>("cart_type"),
        "completed_at":c.get::<Option<chrono::DateTime<chrono::Utc>>,_>("completed_at"),
        "payment_authorized_at":c.get::<Option<chrono::DateTime<chrono::Utc>>,_>("payment_authorized_at"),
        "idempotency_key":c.get::<Option<String>,_>("idempotency_key"),
        "context":c.get::<Option<serde_json::Value>,_>("context"),
        "sales_channel_id":c.get::<Option<Uuid>,_>("sales_channel_id"),
        "metadata":c.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at":c.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at":c.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
        "items":items,"shipping_methods":shipping_methods,
        "discounts":discounts,"gift_cards":[],
        "payment_session":selected_payment_session,"payment_sessions":payment_sessions,"payment":null,
        "subtotal":subtotal,"tax_total":tax_total,
        "shipping_total":shipping_total,"discount_total":discount_total,"gift_card_total":0,"total":total,
    }))
}

// ─── Create cart ──────────────────────────────────────────────────────────────

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let region_id: Uuid = if let Some(rid) = payload.get("region_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()) {
        rid
    } else {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM regions WHERE deleted_at IS NULL LIMIT 1")
            .fetch_optional(&*state.db).await?
            .ok_or_else(|| AppError::BadRequest("region_id required (no default region found)".into()))?
    };
    sqlx::query("INSERT INTO carts (id, region_id, cart_type, created_at, updated_at) VALUES ($1,$2,'default',NOW(),NOW())")
        .bind(id).bind(region_id).execute(&*state.db).await?;
    // If items were provided at creation time add them
    if let Some(arr) = payload.get("items").and_then(|v| v.as_array()) {
        for item in arr {
            if let (Some(vid), Some(qty)) = (item.get("variant_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()), item.get("quantity").and_then(|v| v.as_i64())) {
                let _ = add_item_inner(&state, id, vid, qty as i32).await;
            }
        }
    }
    Ok((StatusCode::CREATED, Json(serde_json::json!({"cart":fetch_cart(&state, id).await?}))))
}

// ─── Get cart ─────────────────────────────────────────────────────────────────

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, id).await?})))
}

// ─── Update cart ──────────────────────────────────────────────────────────────

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    // Handle shipping address
    if let Some(addr) = p.get("shipping_address") {
        let addr_id = Uuid::new_v4();
        sqlx::query("INSERT INTO addresses (id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW(),NOW()) ON CONFLICT (id) DO UPDATE SET first_name=$2, last_name=$3, updated_at=NOW()")
            .bind(addr_id).bind(addr.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("company").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("city").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("province").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
        sqlx::query("UPDATE carts SET shipping_address_id = $1, updated_at = NOW() WHERE id = $2").bind(addr_id).bind(id).execute(&*state.db).await?;
    }
    if let Some(email) = p.get("email").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE carts SET email = $1, updated_at = NOW() WHERE id = $2").bind(email).bind(id).execute(&*state.db).await?;
    }
    if let Some(rid) = p.get("region_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()) {
        sqlx::query("UPDATE carts SET region_id = $1, updated_at = NOW() WHERE id = $2").bind(rid).bind(id).execute(&*state.db).await?;
    }
    if let Some(cid) = p.get("customer_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok()) {
        sqlx::query("UPDATE carts SET customer_id = $1, updated_at = NOW() WHERE id = $2").bind(cid).bind(id).execute(&*state.db).await?;
    }
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, id).await?})))
}

// ─── Line items ───────────────────────────────────────────────────────────────

async fn add_item_inner(state: &AppState, cart_id: Uuid, variant_id: Uuid, quantity: i32) -> Result<(), AppError> {
    // Look up variant for price and title
    let variant = sqlx::query("SELECT title, product_id FROM product_variants WHERE id = $1 AND deleted_at IS NULL")
        .bind(variant_id).fetch_optional(&*state.db).await?;
    let (title, _product_id) = if let Some(v) = variant {
        (v.get::<String,_>("title"), v.get::<Uuid,_>("product_id"))
    } else {
        ("Unknown".to_string(), Uuid::nil())
    };
    // Get unit price — first USD money amount for this variant
    let unit_price: i64 = sqlx::query_scalar("SELECT amount FROM money_amounts WHERE variant_id = $1 AND currency_code = 'usd' AND deleted_at IS NULL AND price_list_id IS NULL LIMIT 1")
        .bind(variant_id).fetch_optional(&*state.db).await?.unwrap_or(0);
    // Check if already in cart → update quantity
    let existing = sqlx::query("SELECT id, quantity FROM line_items WHERE cart_id = $1 AND variant_id = $2 AND order_id IS NULL AND swap_id IS NULL")
        .bind(cart_id).bind(variant_id).fetch_optional(&*state.db).await?;
    if let Some(ex) = existing {
        let new_qty: i32 = ex.get::<i32,_>("quantity") + quantity;
        let lid: Uuid = ex.get("id");
        sqlx::query("UPDATE line_items SET quantity = $1, updated_at = NOW() WHERE id = $2").bind(new_qty).bind(lid).execute(&*state.db).await?;
    } else {
        let lid = Uuid::new_v4();
        sqlx::query("INSERT INTO line_items (id, cart_id, title, is_giftcard, should_merge, allow_discounts, quantity, unit_price, variant_id, created_at, updated_at) VALUES ($1,$2,$3,false,true,true,$4,$5,$6,NOW(),NOW())")
            .bind(lid).bind(cart_id).bind(title).bind(quantity).bind(unit_price).bind(variant_id).execute(&*state.db).await?;
    }
    Ok(())
}

pub async fn add_line_item(State(state): State<AppState>, Path(cart_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let variant_id: Uuid = payload.get("variant_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or_else(|| AppError::BadRequest("variant_id required".into()))?;
    let quantity: i32 = payload.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    add_item_inner(&state, cart_id, variant_id, quantity).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?}))))
}

pub async fn update_line_item(State(state): State<AppState>, Path((cart_id, line_id)): Path<(Uuid,Uuid)>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let qty: i32 = payload.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    if qty == 0 {
        sqlx::query("DELETE FROM line_items WHERE id = $1 AND cart_id = $2").bind(line_id).bind(cart_id).execute(&*state.db).await?;
    } else {
        sqlx::query("UPDATE line_items SET quantity = $1, updated_at = NOW() WHERE id = $2 AND cart_id = $3").bind(qty).bind(line_id).bind(cart_id).execute(&*state.db).await?;
    }
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn remove_line_item(State(state): State<AppState>, Path((cart_id, line_id)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM line_items WHERE id = $1 AND cart_id = $2").bind(line_id).bind(cart_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

// ─── Payment sessions ─────────────────────────────────────────────────────────

pub async fn create_payment_sessions(State(state): State<AppState>, Path(cart_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    // Auto-create a "manual" payment session
    let existing = sqlx::query("SELECT id FROM payment_sessions WHERE cart_id = $1 AND provider_id = 'manual'").bind(cart_id).fetch_optional(&*state.db).await?;
    if existing.is_none() {
        let cart = fetch_cart(&state, cart_id).await?;
        let amount = cart["total"].as_i64().unwrap_or(0);
        let sid = Uuid::new_v4();
        sqlx::query("INSERT INTO payment_sessions (id, cart_id, provider_id, is_selected, is_initiated, status, data, amount, created_at, updated_at) VALUES ($1,$2,'manual',true,false,'pending','{}',  $3,NOW(),NOW())")
            .bind(sid).bind(cart_id).bind(amount).execute(&*state.db).await?;
    }
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn select_payment_session(State(state): State<AppState>, Path(cart_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let provider = payload.get("provider_id").and_then(|v| v.as_str()).unwrap_or("manual");
    sqlx::query("UPDATE payment_sessions SET is_selected = (provider_id = $1), updated_at = NOW() WHERE cart_id = $2").bind(provider).bind(cart_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn delete_payment_session(State(state): State<AppState>, Path((cart_id, provider_id)): Path<(Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM payment_sessions WHERE cart_id = $1 AND provider_id = $2").bind(cart_id).bind(provider_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn refresh_payment_session(State(state): State<AppState>, Path((cart_id, provider_id)): Path<(Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE payment_sessions SET updated_at = NOW() WHERE cart_id = $1 AND provider_id = $2").bind(cart_id).bind(provider_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

// ─── Shipping methods ─────────────────────────────────────────────────────────

pub async fn add_shipping_method(State(state): State<AppState>, Path(cart_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let option_id: Uuid = payload.get("option_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or_else(|| AppError::BadRequest("option_id required".into()))?;
    let option = sqlx::query("SELECT id, price_type, amount FROM shipping_options WHERE id = $1").bind(option_id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Shipping option not found".into()))?;
    let price: i64 = option.get::<Option<i64>, _>("amount").unwrap_or(0);
    // Remove any existing shipping method for same option
    sqlx::query("DELETE FROM shipping_methods WHERE cart_id = $1 AND shipping_option_id = $2").bind(cart_id).bind(option_id).execute(&*state.db).await?;
    let smid = Uuid::new_v4();
    sqlx::query("INSERT INTO shipping_methods (id, cart_id, shipping_option_id, price, data, created_at, updated_at) VALUES ($1,$2,$3,$4,'{}',NOW(),NOW())").bind(smid).bind(cart_id).bind(option_id).bind(price).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

// ─── Taxes ───────────────────────────────────────────────────────────────────

pub async fn calculate_taxes(State(state): State<AppState>, Path(cart_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

// ─── Complete cart (checkout) ────────────────────────────────────────────────

pub async fn complete(State(state): State<AppState>, Path(cart_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cart = fetch_cart(&state, cart_id).await?;

    // Check if already completed
    if cart.get("completed_at").and_then(|v| v.as_str()).is_some() {
        // Already an order — find and return it
        if let Some(order_row) = sqlx::query("SELECT id FROM orders WHERE cart_id = $1").bind(cart_id).fetch_optional(&*state.db).await? {
            let order_id: Uuid = order_row.get("id");
            let order_data = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders WHERE id = $1").bind(order_id).fetch_one(&*state.db).await?;
            return Ok(Json(serde_json::json!({"type":"order","data":{
                "id":order_data.get::<Uuid,_>("id"),
                "status":order_data.get::<String,_>("status"),
                "display_id":order_data.get::<i32,_>("display_id"),
                "customer_id":order_data.get::<Uuid,_>("customer_id"),
                "email":order_data.get::<String,_>("email"),
                "currency_code":order_data.get::<String,_>("currency_code"),
                "total":cart["total"], "subtotal":cart["subtotal"],
                "tax_total":cart["tax_total"], "shipping_total":cart["shipping_total"],
            }})));
        }
    }

    let email = cart.get("email").and_then(|v| v.as_str()).unwrap_or("guest@example.com");
    let region_id: Uuid = cart.get("region_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or_default();
    let customer_id: Uuid = cart.get("customer_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(Uuid::new_v4());
    let currency = sqlx::query_scalar::<_, String>("SELECT currency_code FROM regions WHERE id = $1").bind(region_id).fetch_optional(&*state.db).await?.unwrap_or_else(|| "usd".into());

    let order_id = Uuid::new_v4();
    let display_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(display_id), 0) + 1 FROM orders").fetch_one(&*state.db).await?;
    sqlx::query("INSERT INTO orders (id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, created_at, updated_at) VALUES ($1,'pending','not_fulfilled','awaiting',$2,$3,$4,$5,$6,$7,NOW(),NOW())")
        .bind(order_id).bind(display_id).bind(cart_id).bind(customer_id).bind(email).bind(region_id).bind(currency).execute(&*state.db).await?;

    // Mark cart as completed
    sqlx::query("UPDATE carts SET completed_at = NOW(), updated_at = NOW() WHERE id = $1").bind(cart_id).execute(&*state.db).await?;

    // Move line items to the order
    sqlx::query("UPDATE line_items SET order_id = $1, updated_at = NOW() WHERE cart_id = $2 AND order_id IS NULL").bind(order_id).bind(cart_id).execute(&*state.db).await?;

    Ok(Json(serde_json::json!({"type":"order","data":{
        "id":order_id, "status":"pending",
        "fulfillment_status":"not_fulfilled", "payment_status":"awaiting",
        "display_id":display_id, "cart_id":cart_id,
        "customer_id":customer_id, "email":email,
        "region_id":region_id,
        "total":cart["total"], "subtotal":cart["subtotal"],
        "tax_total":cart["tax_total"], "shipping_total":cart["shipping_total"],
        "items":cart["items"],"shipping_methods":cart["shipping_methods"],
    }})))
}

// ─── Discounts ────────────────────────────────────────────────────────────────

pub async fn apply_discount(State(state): State<AppState>, Path(cart_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let code = payload.get("code").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("code required".into()))?;
    let discount = sqlx::query("SELECT id, code, is_disabled FROM discounts WHERE code = $1 AND deleted_at IS NULL").bind(code).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound(format!("Discount '{}' not found", code)))?;
    if discount.get::<bool,_>("is_disabled") { return Err(AppError::BadRequest("Discount is disabled".into())); }
    let did: Uuid = discount.get("id");
    // insert into cart_discounts if not already
    sqlx::query("INSERT INTO cart_discounts (cart_id, discount_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(cart_id).bind(did).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn remove_discount(State(state): State<AppState>, Path((cart_id, code)): Path<(Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> {
    // look up discount id by code
    if let Some(d) = sqlx::query("SELECT id FROM discounts WHERE code = $1 AND deleted_at IS NULL")
        .bind(&code).fetch_optional(&*state.db).await? {
        let did: Uuid = d.get("id");
        sqlx::query("DELETE FROM cart_discounts WHERE cart_id = $1 AND discount_id = $2")
            .bind(cart_id).bind(did).execute(&*state.db).await?;
    }
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}

pub async fn update_payment_session(State(state): State<AppState>, Path((cart_id, provider_id)): Path<(Uuid, String)>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE payment_sessions SET updated_at = NOW() WHERE cart_id = $1 AND provider_id = $2").bind(cart_id).bind(provider_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"cart":fetch_cart(&state, cart_id).await?})))
}
