//! Store cart handlers — MedusaJS v1 compatible
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

async fn fetch_cart(state: &AppState, id: Uuid) -> Result<serde_json::Value, AppError> {
    let c = sqlx::query!(
        "SELECT id, email, billing_address_id, shipping_address_id, region_id, customer_id, payment_id, cart_type, completed_at, payment_authorized_at, idempotency_key, context, sales_channel_id, metadata, created_at, updated_at, deleted_at FROM carts WHERE id = $1 AND deleted_at IS NULL",
        id
    ).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Cart not found".into()))?;

    let items = sqlx::query!(
        "SELECT id, cart_id, order_id, title, description, thumbnail, is_return, is_giftcard, should_merge, allow_discounts, has_shipping, unit_price, variant_id, quantity, fulfilled_quantity, returned_quantity, shipped_quantity, metadata, created_at, updated_at FROM line_items WHERE cart_id = $1",
        id
    ).fetch_all(&*state.db).await?
    .into_iter()
    .map(|i| serde_json::json!({
        "id":i.id,"cart_id":i.cart_id,"title":i.title,"description":i.description,
        "thumbnail":i.thumbnail,"is_return":i.is_return,"is_giftcard":i.is_giftcard,
        "should_merge":i.should_merge,"allow_discounts":i.allow_discounts,
        "has_shipping":i.has_shipping,"unit_price":i.unit_price,"variant_id":i.variant_id,
        "quantity":i.quantity,"fulfilled_quantity":i.fulfilled_quantity,
        "returned_quantity":i.returned_quantity,"shipped_quantity":i.shipped_quantity,
        "metadata":i.metadata,"created_at":i.created_at,"updated_at":i.updated_at,
        "adjustments":[],"tax_lines":[],"subtotal":i.unit_price * i.quantity as i64,
        "discount_total":0,"tax_total":0,"total":i.unit_price * i.quantity as i64,
    }))
    .collect::<Vec<_>>();

    let subtotal: i64 = items.iter().map(|i| i.get("subtotal").and_then(|v| v.as_i64()).unwrap_or(0)).sum();

    let region = sqlx::query!(
        "SELECT id, name, currency_code, tax_rate, gift_cards_taxable, automatic_taxes, metadata, created_at, updated_at FROM regions WHERE id = $1",
        c.region_id
    ).fetch_optional(&*state.db).await?
    .map(|r| serde_json::json!({"id":r.id,"name":r.name,"currency_code":r.currency_code,"tax_rate":r.tax_rate,"gift_cards_taxable":r.gift_cards_taxable,"automatic_taxes":r.automatic_taxes,"countries":[],"payment_providers":[],"fulfillment_providers":[]}));

    let payment_sessions = sqlx::query!(
        "SELECT id, cart_id, provider_id, is_selected, is_initiated, status, data, amount, payment_authorized_at, created_at, updated_at FROM payment_sessions WHERE cart_id = $1",
        id
    ).fetch_all(&*state.db).await?
    .into_iter()
    .map(|s| serde_json::json!({"id":s.id,"cart_id":s.cart_id,"provider_id":s.provider_id,"is_selected":s.is_selected,"is_initiated":s.is_initiated,"status":s.status,"data":s.data,"amount":s.amount,"payment_authorized_at":s.payment_authorized_at}))
    .collect::<Vec<_>>();

    let selected_session = payment_sessions.iter().find(|s| s.get("is_selected").and_then(|v| v.as_bool()).unwrap_or(false)).cloned();

    let shipping_methods = sqlx::query!(
        "SELECT id, shipping_option_id, cart_id, price, data FROM shipping_methods WHERE cart_id = $1",
        id
    ).fetch_all(&*state.db).await?
    .into_iter()
    .map(|s| serde_json::json!({"id":s.id,"shipping_option_id":s.shipping_option_id,"cart_id":s.cart_id,"price":s.price,"data":s.data}))
    .collect::<Vec<_>>();

    let shipping_total: i64 = shipping_methods.iter().map(|s| s.get("price").and_then(|v| v.as_i64()).unwrap_or(0)).sum();
    let total = subtotal + shipping_total;

    Ok(serde_json::json!({
        "id": c.id,
        "email": c.email,
        "billing_address_id": c.billing_address_id,
        "billing_address": null,
        "shipping_address_id": c.shipping_address_id,
        "shipping_address": null,
        "items": items,
        "region_id": c.region_id,
        "region": region,
        "discounts": [],
        "gift_cards": [],
        "customer_id": c.customer_id,
        "customer": null,
        "payment_session": selected_session,
        "payment_sessions": payment_sessions,
        "payment": null,
        "shipping_methods": shipping_methods,
        "type": c.cart_type,
        "completed_at": c.completed_at,
        "payment_authorized_at": c.payment_authorized_at,
        "idempotency_key": c.idempotency_key,
        "context": c.context,
        "sales_channel_id": c.sales_channel_id,
        "metadata": c.metadata,
        "created_at": c.created_at,
        "updated_at": c.updated_at,
        "deleted_at": c.deleted_at,
        "subtotal": subtotal,
        "discount_total": 0,
        "shipping_total": shipping_total,
        "tax_total": 0,
        "gift_card_total": 0,
        "total": total,
    }))
}

pub async fn get_cart(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}

pub async fn create_cart(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let region_id: Uuid = payload.get("region_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("region_id is required".into()))?;

    sqlx::query!(
        "INSERT INTO carts (id, region_id, cart_type, context, created_at, updated_at) VALUES ($1,$2,'default',$3,NOW(),NOW())",
        id, region_id, payload.get("context").cloned()
    ).execute(&*state.db).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? }))))
}

pub async fn update_cart(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    // Handle region_id, email, billing_address, shipping_address, customer_id, discounts, gift_cards
    let region_id: Option<Uuid> = payload.get("region_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
    let email = payload.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
    let customer_id: Option<Uuid> = payload.get("customer_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());

    // Upsert shipping address
    let shipping_address_id: Option<Uuid> = if let Some(addr) = payload.get("shipping_address") {
        let aid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO addresses (id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW(),NOW()) ON CONFLICT DO NOTHING RETURNING id",
            aid,
            addr.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("company").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("city").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("province").and_then(|v| v.as_str()).map(|s| s.to_string()),
            addr.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
        ).fetch_optional(&*state.db).await?.map(|r| r.id)
    } else { None };

    sqlx::query!(
        "UPDATE carts SET region_id = COALESCE($2, region_id), email = COALESCE($3, email), customer_id = COALESCE($4, customer_id), shipping_address_id = COALESCE($5, shipping_address_id), updated_at = NOW() WHERE id = $1",
        id, region_id, email, customer_id, shipping_address_id
    ).execute(&*state.db).await?;

    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}

pub async fn add_line_item(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let variant_id: Uuid = payload.get("variant_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or_else(|| AppError::BadRequest("variant_id required".into()))?;
    let quantity = payload.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

    let variant = sqlx::query!(
        "SELECT pv.id, pv.title, pv.inventory_quantity, pv.allow_backorder, p.title as product_title, p.thumbnail FROM product_variants pv JOIN products p ON p.id = pv.product_id WHERE pv.id = $1 AND pv.deleted_at IS NULL",
        variant_id
    ).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Variant not found".into()))?;

    if !variant.allow_backorder && variant.inventory_quantity < quantity {
        return Err(AppError::BadRequest(format!("Not enough stock. Available: {}", variant.inventory_quantity)));
    }

    // Find the price for the cart region
    let cart = sqlx::query!("SELECT region_id FROM carts WHERE id = $1", id).fetch_one(&*state.db).await?;
    let price = sqlx::query!(
        "SELECT amount FROM money_amounts WHERE variant_id = $1 AND (region_id = $2 OR region_id IS NULL) AND deleted_at IS NULL ORDER BY region_id DESC NULLS LAST LIMIT 1",
        variant_id, cart.region_id
    ).fetch_optional(&*state.db).await?.map(|r| r.amount).unwrap_or(0);

    // Merge with existing item if possible
    let existing = sqlx::query!(
        "SELECT id, quantity FROM line_items WHERE cart_id = $1 AND variant_id = $2",
        id, variant_id
    ).fetch_optional(&*state.db).await?;

    if let Some(ex) = existing {
        sqlx::query!("UPDATE line_items SET quantity = $1, updated_at = NOW() WHERE id = $2", ex.quantity + quantity, ex.id).execute(&*state.db).await?;
    } else {
        let item_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO line_items (id, cart_id, title, thumbnail, unit_price, variant_id, quantity, is_return, is_giftcard, should_merge, allow_discounts, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,false,false,true,true,NOW(),NOW())",
            item_id, id, variant.product_title, variant.thumbnail, price, variant_id, quantity
        ).execute(&*state.db).await?;
    }

    Ok((StatusCode::OK, Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? }))))
}

pub async fn update_line_item(State(state): State<AppState>, Path((cart_id, line_id)): Path<(Uuid, Uuid)>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let quantity = payload.get("quantity").and_then(|v| v.as_i64()).map(|q| q as i32);
    if let Some(qty) = quantity {
        if qty == 0 {
            sqlx::query!("DELETE FROM line_items WHERE id = $1 AND cart_id = $2", line_id, cart_id).execute(&*state.db).await?;
        } else {
            sqlx::query!("UPDATE line_items SET quantity = $1, updated_at = NOW() WHERE id = $2 AND cart_id = $3", qty, line_id, cart_id).execute(&*state.db).await?;
        }
    }
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, cart_id).await? })))
}

pub async fn delete_line_item(State(state): State<AppState>, Path((cart_id, line_id)): Path<(Uuid, Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("DELETE FROM line_items WHERE id = $1 AND cart_id = $2", line_id, cart_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, cart_id).await? })))
}

pub async fn create_payment_sessions(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    // Create a mock manual payment session
    let cart = fetch_cart(&state, id).await?;
    let total = cart.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let session_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO payment_sessions (id, cart_id, provider_id, status, is_initiated, is_selected, amount, data, created_at, updated_at) VALUES ($1,$2,'manual','pending',true,false,$3,'{}',NOW(),NOW()) ON CONFLICT DO NOTHING",
        session_id, id, total
    ).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}

pub async fn select_payment_session(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let provider_id = payload.get("provider_id").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("provider_id required".into()))?;
    sqlx::query!("UPDATE payment_sessions SET is_selected = false WHERE cart_id = $1", id).execute(&*state.db).await?;
    sqlx::query!("UPDATE payment_sessions SET is_selected = true WHERE cart_id = $1 AND provider_id = $2", id, provider_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}

pub async fn delete_payment_session(State(state): State<AppState>, Path((cart_id, provider_id)): Path<(Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("DELETE FROM payment_sessions WHERE cart_id = $1 AND provider_id = $2", cart_id, provider_id).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, cart_id).await? })))
}

pub async fn add_shipping_method(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let option_id: Uuid = payload.get("option_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or_else(|| AppError::BadRequest("option_id required".into()))?;
    let option = sqlx::query!("SELECT id, name, amount FROM shipping_options WHERE id = $1 AND deleted_at IS NULL", option_id)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Shipping option not found".into()))?;
    // Remove existing methods for this cart
    sqlx::query!("DELETE FROM shipping_methods WHERE cart_id = $1", id).execute(&*state.db).await?;
    let method_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO shipping_methods (id, cart_id, shipping_option_id, price, data) VALUES ($1,$2,$3,$4,'{}')",
        method_id, id, option_id, option.amount.unwrap_or(0)
    ).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}

pub async fn complete_cart(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cart = fetch_cart(&state, id).await?;
    let customer_id: Option<Uuid> = cart.get("customer_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
    let region_id: Uuid = cart.get("region_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or_default();
    let email = cart.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let total = cart.get("total").and_then(|v| v.as_i64()).unwrap_or(0);

    // Check we have email
    if email.is_empty() { return Err(AppError::BadRequest("Cart must have an email before completing".into())); }

    let order_id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;

    let display_id = sqlx::query_scalar!("SELECT COALESCE(MAX(display_id), 0) + 1 FROM orders")
        .fetch_one(&mut *tx).await?.unwrap_or(1);

    sqlx::query!(
        "INSERT INTO orders (id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, created_at, updated_at) SELECT $1,'pending','not_fulfilled','awaiting',$2,$3,$4,$5,$6,r.currency_code,NOW(),NOW() FROM regions r WHERE r.id = $6",
        order_id, display_id, id, customer_id.unwrap_or(Uuid::new_v4()), email, region_id
    ).execute(&mut *tx).await?;

    // Move line items to order
    sqlx::query!("UPDATE line_items SET order_id = $1, cart_id = NULL, updated_at = NOW() WHERE cart_id = $2", order_id, id).execute(&mut *tx).await?;
    // Move shipping methods to order
    sqlx::query!("UPDATE shipping_methods SET order_id = $1, cart_id = NULL WHERE cart_id = $2", order_id, id).execute(&mut *tx).await?;
    // Mark cart completed
    sqlx::query!("UPDATE carts SET completed_at = NOW(), updated_at = NOW() WHERE id = $1", id).execute(&mut *tx).await?;

    tx.commit().await?;

    let order = sqlx::query!(
        "SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, created_at, updated_at FROM orders WHERE id = $1",
        order_id
    ).fetch_one(&*state.db).await?;

    Ok(Json(serde_json::json!({
        "type": "order",
        "order": {
            "id": order.id,
            "status": order.status,
            "fulfillment_status": order.fulfillment_status,
            "payment_status": order.payment_status,
            "display_id": order.display_id,
            "cart_id": order.cart_id,
            "customer_id": order.customer_id,
            "email": order.email,
            "region_id": order.region_id,
            "currency_code": order.currency_code,
            "created_at": order.created_at,
            "updated_at": order.updated_at,
            "items": [],
            "shipping_methods": [],
            "payments": [],
            "subtotal": total,
            "tax_total": 0,
            "shipping_total": 0,
            "discount_total": 0,
            "total": total,
        }
    })))
}

pub async fn calculate_taxes(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    // Return cart with tax fields populated
    Ok(Json(serde_json::json!({ "cart": fetch_cart(&state, id).await? })))
}
