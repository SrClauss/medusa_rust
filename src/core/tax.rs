//! Tax calculation engine.
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaxRate {
    pub id: Uuid,
    pub name: String,
    pub rate: Option<f64>,
    pub region_id: Uuid,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    pub name: String,
    pub code: Option<String>,
    pub rate: f64,
    pub amount: i64,
}

pub fn calculate_tax_lines(subtotal: i64, rates: &[TaxRate]) -> Vec<TaxLine> {
    rates.iter().map(|r| TaxLine {
        name: r.name.clone(),
        code: r.code.clone(),
        rate: r.rate.unwrap_or(0.0),
        amount: ((subtotal as f64) * r.rate.unwrap_or(0.0) / 100.0).round() as i64,
    }).collect()
}

pub fn total_tax(lines: &[TaxLine]) -> i64 { lines.iter().map(|l| l.amount).sum() }

pub async fn get_region_tax_rates(pool: &PgPool, region_id: Uuid) -> Result<Vec<TaxRate>, AppError> {
    Ok(sqlx::query_as::<_, TaxRate>("SELECT id, name, rate, region_id, code FROM tax_rates WHERE region_id = $1")
        .bind(region_id).fetch_all(pool).await?)
}
