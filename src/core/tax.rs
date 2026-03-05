//! Tax calculation engine.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// A tax rate associated with a region or product type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRate {
    pub id: Uuid,
    pub name: String,
    /// Rate as a percentage (e.g. 20.0 means 20 %).
    pub rate: f64,
    pub region_id: Uuid,
    pub code: Option<String>,
}

/// Tax line applied to a single line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    pub name: String,
    pub code: Option<String>,
    pub rate: f64,
    /// Tax amount in the smallest currency unit (cents).
    pub amount: i64,
}

/// Calculates tax lines for a given subtotal and a list of applicable rates.
pub fn calculate_tax_lines(subtotal: i64, rates: &[TaxRate]) -> Vec<TaxLine> {
    rates
        .iter()
        .map(|r| TaxLine {
            name: r.name.clone(),
            code: r.code.clone(),
            rate: r.rate,
            amount: ((subtotal as f64) * r.rate / 100.0).round() as i64,
        })
        .collect()
}

/// Sums all tax amounts in a list of tax lines.
pub fn total_tax(lines: &[TaxLine]) -> i64 {
    lines.iter().map(|l| l.amount).sum()
}

/// Fetches applicable tax rates for a region from the database.
pub async fn get_region_tax_rates(
    pool: &PgPool,
    region_id: Uuid,
) -> Result<Vec<TaxRate>, AppError> {
    let rows = sqlx::query_as!(
        TaxRate,
        r#"SELECT id, name, rate, region_id, code
           FROM tax_rates
           WHERE region_id = $1"#,
        region_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
