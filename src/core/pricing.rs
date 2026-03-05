//! Pricing engine — resolves the best price for a variant given context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single money amount for a variant in a specific currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyAmount {
    pub id: Uuid,
    pub variant_id: Uuid,
    pub region_id: Option<Uuid>,
    pub price_list_id: Option<Uuid>,
    pub currency_code: String,
    /// Amount in the smallest currency unit (cents).
    pub amount: i64,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
}

/// Context used to select the best price.
#[derive(Debug, Clone)]
pub struct PricingContext {
    pub region_id: Uuid,
    pub currency_code: String,
    pub customer_id: Option<Uuid>,
    pub quantity: i32,
}

/// Selects the best applicable price from a list of money amounts.
///
/// Priority:
/// 1. Price list prices that match quantity range (sale / override)
/// 2. Region-specific default prices
/// 3. Default prices without a region
pub fn resolve_price(amounts: &[MoneyAmount], ctx: &PricingContext) -> Option<i64> {
    // Price-list entries (highest priority) — match quantity range.
    let price_list_price = amounts
        .iter()
        .filter(|a| {
            a.price_list_id.is_some()
                && a.currency_code == ctx.currency_code
                && a.min_quantity.map_or(true, |min| ctx.quantity >= min)
                && a.max_quantity.map_or(true, |max| ctx.quantity <= max)
        })
        .map(|a| a.amount)
        .min();

    if let Some(p) = price_list_price {
        return Some(p);
    }

    // Region-specific price.
    let region_price = amounts
        .iter()
        .filter(|a| {
            a.price_list_id.is_none()
                && a.region_id == Some(ctx.region_id)
                && a.currency_code == ctx.currency_code
        })
        .map(|a| a.amount)
        .min();

    if let Some(p) = region_price {
        return Some(p);
    }

    // Default (currency-only) price.
    amounts
        .iter()
        .filter(|a| {
            a.price_list_id.is_none()
                && a.region_id.is_none()
                && a.currency_code == ctx.currency_code
        })
        .map(|a| a.amount)
        .min()
}

/// Applies a percentage discount to an amount.
pub fn apply_percentage_discount(amount: i64, percentage: f64) -> i64 {
    (amount as f64 * (1.0 - percentage / 100.0)).round() as i64
}

/// Applies a fixed discount, clamped to zero.
pub fn apply_fixed_discount(amount: i64, discount: i64) -> i64 {
    (amount - discount).max(0)
}
