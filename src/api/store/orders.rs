//! Store order handlers
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct OrderParams {
    pub cart_id: Option<Uuid>,
    pub display_id: Option<i32>,
    pub email: Option<String>,
}

async fn build_order(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let items_rows = sqlx::query(
        "SELECT id, title, description, thumbnail, unit_price, quantity, \
         fulfilled_quantity, returned_quantity, shipped_quantity, variant_id, metadata, created_at, updated_at \
         FROM line_items WHERE order_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default();

    let mut subtotal: i64 = 0;
    let items = items_rows
        .iter()
        .map(|i| {
            let unit_price = i.get::<i64, _>("unit_price");
            let quantity = i.get::<i32, _>("quantity");
            subtotal += unit_price * (quantity as i64);
            serde_json::json!({
                "id": i.get::<Uuid, _>("id"),
                "title": i.get::<String, _>("title"),
                "description": i.get::<Option<String>, _>("description"),
                "thumbnail": i.get::<Option<String>, _>("thumbnail"),
                "unit_price": unit_price,
                "quantity": quantity,
                "fulfilled_quantity": i.get::<Option<i32>, _>("fulfilled_quantity"),
                "returned_quantity": i.get::<Option<i32>, _>("returned_quantity"),
                "shipped_quantity": i.get::<Option<i32>, _>("shipped_quantity"),
                "variant_id": i.get::<Option<Uuid>, _>("variant_id"),
                "subtotal": unit_price * (quantity as i64),
                "metadata": i.get::<Option<serde_json::Value>, _>("metadata"),
                "created_at": i.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "updated_at": i.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
                "adjustments": [],
                "tax_lines": [],
            })
        })
        .collect::<Vec<_>>();

    let tax_rate = r.get::<Option<f64>, _>("tax_rate").unwrap_or(0.0);
    let tax_total = ((subtotal as f64) * tax_rate / 100.0) as i64;

    let shipping_methods_rows = sqlx::query(
        "SELECT sm.id, sm.price, sm.shipping_option_id, so.name as shipping_option_name FROM shipping_methods sm \
         LEFT JOIN shipping_options so ON so.id = sm.shipping_option_id \
         WHERE sm.order_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default();

    let mut shipping_total: i64 = 0;
    let shipping_methods = shipping_methods_rows
        .iter()
        .map(|sm| {
            let price = sm.get::<i64, _>("price");
            shipping_total += price;
            serde_json::json!({
                "id": sm.get::<Uuid, _>("id"),
                "price": price,
                "shipping_option_id": sm.get::<Uuid, _>("shipping_option_id"),
                "shipping_option": { "name": sm.get::<Option<String>, _>("shipping_option_name") },
                "tax_lines": [],
            })
        })
        .collect::<Vec<_>>();

    // Resolve addresses
    let shipping_address = if let Some(aid) = r.get::<Option<Uuid>, _>("shipping_address_id") {
        sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code FROM addresses WHERE id = $1")
            .bind(aid).fetch_optional(&*state.db).await.ok().flatten()
            .map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")}))
    } else { None };
    let billing_address = if let Some(aid) = r.get::<Option<Uuid>, _>("billing_address_id") {
        sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code FROM addresses WHERE id = $1")
            .bind(aid).fetch_optional(&*state.db).await.ok().flatten()
            .map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")}))
    } else { None };

    let region_id: Uuid = r.get("region_id");
    let region = sqlx::query("SELECT id, name, currency_code, tax_rate FROM regions WHERE id = $1")
        .bind(region_id).fetch_optional(&*state.db).await.ok().flatten()
        .map(|rr| serde_json::json!({"id":rr.get::<Uuid,_>("id"),"name":rr.get::<String,_>("name"),"currency_code":rr.get::<String,_>("currency_code"),"tax_rate":rr.get::<f64,_>("tax_rate")}));

    let total = subtotal + tax_total + shipping_total;

    Ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "object": "order",
        "status": r.get::<String, _>("status"),
        "fulfillment_status": r.get::<String, _>("fulfillment_status"),
        "payment_status": r.get::<String, _>("payment_status"),
        "display_id": r.get::<i32, _>("display_id"),
        "cart_id": r.get::<Option<Uuid>, _>("cart_id"),
        "customer_id": r.get::<Uuid, _>("customer_id"),
        "email": r.get::<String, _>("email"),
        "region_id": region_id,
        "region": region,
        "currency_code": r.get::<String, _>("currency_code"),
        "tax_rate": tax_rate,
        "shipping_address_id": r.get::<Option<Uuid>, _>("shipping_address_id"),
        "shipping_address": shipping_address,
        "billing_address_id": r.get::<Option<Uuid>, _>("billing_address_id"),
        "billing_address": billing_address,
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "canceled_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "items": items,
        "shipping_methods": shipping_methods,
        "payments": [],
        "fulfillments": [],
        "returns": [],
        "discounts": [],
        "gift_cards": [],
        "subtotal": subtotal,
        "tax_total": tax_total,
        "shipping_total": shipping_total,
        "discount_total": 0,
        "gift_card_total": 0,
        "total": total,
    }))
}

pub async fn get_order(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, \
         customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, \
         metadata, created_at, updated_at, canceled_at \
         FROM orders WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Order not found".into()))?;
    Ok(Json(serde_json::json!({ "order": build_order(&state, id, &r).await? })))
}

pub async fn get_order_by_params(State(state): State<AppState>, Query(p): Query<OrderParams>) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(cart_id) = p.cart_id {
        let r = sqlx::query(
            "SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, \
             customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, \
             metadata, created_at, updated_at, canceled_at \
             FROM orders WHERE cart_id = $1",
        )
        .bind(cart_id)
        .fetch_optional(&*state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Order not found".into()))?;
        let id: Uuid = r.get("id");
        return Ok(Json(serde_json::json!({ "order": build_order(&state, id, &r).await? })));
    }
    if let (Some(did), Some(email)) = (p.display_id, p.email.as_ref()) {
        let r = sqlx::query(
            "SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, \
             customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, \
             metadata, created_at, updated_at, canceled_at \
             FROM orders WHERE display_id = $1 AND email = $2",
        )
        .bind(did)
        .bind(email)
        .fetch_optional(&*state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Order not found".into()))?;
        let id: Uuid = r.get("id");
        return Ok(Json(serde_json::json!({ "order": build_order(&state, id, &r).await? })));
    }
    Err(AppError::BadRequest("Provide cart_id, or display_id + email".into()))
}

pub async fn get_orders_batch(State(state): State<AppState>, Query(p): Query<OrderParams>) -> Result<Json<serde_json::Value>, AppError> {
    get_order_by_params(axum::extract::State(state), axum::extract::Query(p)).await
}
