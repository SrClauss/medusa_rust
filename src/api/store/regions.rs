//! Store regions handlers
use axum::{{extract::{{Path, State}}, Json}};
use sqlx::Row;
use uuid::Uuid;
use crate::{{error::AppError, state::AppState}};

pub async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {{
    let rows = sqlx::query("SELECT id, name, currency_code, tax_rate, gift_cards_taxable, automatic_taxes, metadata, created_at, updated_at FROM regions WHERE deleted_at IS NULL ORDER BY name")
        .fetch_all(&*state.db).await?;
    let regions: Vec<_> = rows.iter().map(|r| serde_json::json!({{
        "id": r.get::<Uuid,_>("id"), "name": r.get::<String,_>("name"),
        "currency_code": r.get::<String,_>("currency_code"),
        "tax_rate": r.get::<f64,_>("tax_rate"),
        "gift_cards_taxable": r.get::<bool,_>("gift_cards_taxable"),
        "automatic_taxes": r.get::<bool,_>("automatic_taxes"),
        "metadata": r.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
        "countries": [], "payment_providers": [], "fulfillment_providers": [], "tax_rates": [],
        "currency": {{"code": r.get::<String,_>("currency_code"), "symbol": "", "symbol_native": "", "name": ""}},
    }})).collect();
    Ok(Json(serde_json::json!({{"regions": regions, "count": regions.len()}})))
}}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {{
    let r = sqlx::query("SELECT id, name, currency_code, tax_rate, gift_cards_taxable, automatic_taxes, metadata, created_at, updated_at FROM regions WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Region not found".into()))?;
    let countries = sqlx::query("SELECT id, iso_2, iso_3, num_code, name, display_name FROM countries WHERE region_id = $1")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|c| serde_json::json!({{
            "id": c.get::<Uuid,_>("id"), "iso_2": c.get::<String,_>("iso_2"),
            "iso_3": c.get::<String,_>("iso_3"),
            "name": c.get::<String,_>("name"),
            "display_name": c.get::<String,_>("display_name"),
        }})).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({{"region": {{
        "id": r.get::<Uuid,_>("id"), "name": r.get::<String,_>("name"),
        "currency_code": r.get::<String,_>("currency_code"),
        "tax_rate": r.get::<f64,_>("tax_rate"),
        "gift_cards_taxable": r.get::<bool,_>("gift_cards_taxable"),
        "automatic_taxes": r.get::<bool,_>("automatic_taxes"),
        "metadata": r.get::<Option<serde_json::Value>,_>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
        "countries": countries, "payment_providers": [], "fulfillment_providers": [], "tax_rates": [],
    }}}})))
}}
