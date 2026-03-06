//! Admin order handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

async fn build_order(state: &AppState, id: Uuid, r: &sqlx::postgres::PgRow) -> Result<serde_json::Value, AppError> {
    let items_rows = sqlx::query(
        "SELECT id, title, description, thumbnail, unit_price, quantity, \
         fulfilled_quantity, returned_quantity, shipped_quantity, variant_id, metadata, created_at, updated_at \
         FROM line_items WHERE order_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await?;

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

    let fulfillments = sqlx::query(
        "SELECT id, provider_id, tracking_numbers, shipped_at, canceled_at, created_at, updated_at \
         FROM fulfillments WHERE order_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|f| serde_json::json!({
        "id": f.get::<Uuid, _>("id"),
        "provider_id": f.get::<String, _>("provider_id"),
        "tracking_numbers": f.get::<Option<serde_json::Value>, _>("tracking_numbers"),
        "shipped_at": f.get::<Option<chrono::DateTime<chrono::Utc>>, _>("shipped_at"),
        "canceled_at": f.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "created_at": f.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": f.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "items": [],
        "tracking_links": [],
    }))
    .collect::<Vec<_>>();

    let payments = sqlx::query(
        "SELECT id, amount, currency_code, provider_id, captured_at, cancelled_at, created_at, updated_at \
         FROM payments WHERE order_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|p| serde_json::json!({
        "id": p.get::<Uuid, _>("id"),
        "amount": p.get::<i64, _>("amount"),
        "currency_code": p.get::<String, _>("currency_code"),
        "provider_id": p.get::<String, _>("provider_id"),
        "captured_at": p.get::<Option<chrono::DateTime<chrono::Utc>>, _>("captured_at"),
        "cancelled_at": p.get::<Option<chrono::DateTime<chrono::Utc>>, _>("cancelled_at"),
        "created_at": p.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": p.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }))
    .collect::<Vec<_>>();

    // Resolve shipping and billing addresses
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

    // Resolve region
    let region_id: Uuid = r.get("region_id");
    let region = sqlx::query("SELECT id, name, currency_code, tax_rate FROM regions WHERE id = $1")
        .bind(region_id).fetch_optional(&*state.db).await.ok().flatten()
        .map(|rr| serde_json::json!({"id":rr.get::<Uuid,_>("id"),"name":rr.get::<String,_>("name"),"currency_code":rr.get::<String,_>("currency_code"),"tax_rate":rr.get::<f64,_>("tax_rate")}));

    // Resolve customer
    let customer_id: Uuid = r.get("customer_id");
    let customer = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account FROM customers WHERE id = $1 AND deleted_at IS NULL")
        .bind(customer_id).fetch_optional(&*state.db).await.ok().flatten()
        .map(|c| serde_json::json!({"id":c.get::<Uuid,_>("id"),"email":c.get::<String,_>("email"),"first_name":c.get::<Option<String>,_>("first_name"),"last_name":c.get::<Option<String>,_>("last_name"),"phone":c.get::<Option<String>,_>("phone"),"has_account":c.get::<bool,_>("has_account")}));

    let total = subtotal + tax_total + shipping_total;

    Ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "object": "order",
        "status": r.get::<String, _>("status"),
        "fulfillment_status": r.get::<String, _>("fulfillment_status"),
        "payment_status": r.get::<String, _>("payment_status"),
        "display_id": r.get::<i32, _>("display_id"),
        "cart_id": r.get::<Option<Uuid>, _>("cart_id"),
        "customer_id": customer_id,
        "customer": customer,
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
        "payments": payments,
        "fulfillments": fulfillments,
        "returns": [],
        "claims": [],
        "refunds": [],
        "swaps": [],
        "discounts": [],
        "gift_cards": [],
        "subtotal": subtotal,
        "tax_total": tax_total,
        "shipping_total": shipping_total,
        "discount_total": 0,
        "gift_card_total": 0,
        "gift_card_tax_total": 0,
        "refunded_total": 0,
        "total": total,
        "paid_total": 0,
        "refundable_amount": 0,
    }))
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, metadata, created_at, updated_at, canceled_at FROM orders ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders").fetch_one(&*state.db).await?;
    let mut orders = Vec::new();
    for r in &rows {
        orders.push(build_order(&state, r.get("id"), r).await?);
    }
    Ok(Json(serde_json::json!({"orders":orders,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, metadata, created_at, updated_at, canceled_at FROM orders WHERE id = $1")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
    Ok(Json(serde_json::json!({"order": build_order(&state, id, &r).await?})))
}

pub async fn create(_: State<AppState>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"order":{}})))) }

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET metadata = COALESCE($2, metadata), updated_at = NOW() WHERE id = $1").bind(id).bind(payload.get("metadata").cloned()).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn cancel(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'canceled', canceled_at = NOW(), updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn complete(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'completed', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn archive(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'archived', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn create_fulfillment(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let fid = Uuid::new_v4();
    let provider = payload.get("provider_id").and_then(|v| v.as_str()).unwrap_or("manual");
    sqlx::query("INSERT INTO fulfillments (id, order_id, provider_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())").bind(fid).bind(id).bind(provider).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await.map(|r| (StatusCode::OK, r))
}

pub async fn cancel_fulfillment(State(state): State<AppState>, Path((order_id, fulfillment_id)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE fulfillments SET canceled_at = NOW(), updated_at = NOW() WHERE id = $1 AND order_id = $2").bind(fulfillment_id).bind(order_id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(order_id)).await
}

pub async fn create_shipment(State(state): State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET fulfillment_status = 'shipped', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn create_refund(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"refund":{"id":Uuid::new_v4(),"order_id":id}}))) }
pub async fn request_return(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"return":{"id":Uuid::new_v4(),"order_id":id,"status":"requested"}}))) }
pub async fn create_swap(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"swap":{"id":Uuid::new_v4(),"order_id":id}})))) }
pub async fn create_claim(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"claim_order":{"id":Uuid::new_v4(),"order_id":id}})))) }
