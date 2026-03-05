//! Cart business logic — line-item management, totals calculation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single cart line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub variant_id: Uuid,
    pub title: String,
    pub quantity: i32,
    /// Unit price in the smallest currency unit (e.g. cents).
    pub unit_price: i64,
}

/// Cart totals computed on demand (never stored in the DB directly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartTotals {
    pub subtotal: i64,
    pub discount_total: i64,
    pub tax_total: i64,
    pub shipping_total: i64,
    pub total: i64,
}

/// Computes cart totals from a list of line items.
///
/// `discount_total`, `tax_total` and `shipping_total` should be supplied
/// by the caller after calling the respective engines.
pub fn compute_totals(
    items: &[LineItem],
    discount_total: i64,
    tax_total: i64,
    shipping_total: i64,
) -> CartTotals {
    let subtotal: i64 = items
        .iter()
        .map(|i| i.unit_price * i.quantity as i64)
        .sum();

    CartTotals {
        subtotal,
        discount_total,
        tax_total,
        shipping_total,
        total: subtotal - discount_total + tax_total + shipping_total,
    }
}

/// Validates that all items in the cart have sufficient inventory.
///
/// Returns a list of variant IDs that are out of stock or have insufficient
/// quantity.
pub fn validate_inventory(items: &[LineItem], stock: &[(Uuid, i32)]) -> Vec<Uuid> {
    let stock_map: std::collections::HashMap<Uuid, i32> = stock.iter().cloned().collect();
    items
        .iter()
        .filter(|item| {
            stock_map
                .get(&item.variant_id)
                .map_or(true, |&qty| qty < item.quantity)
        })
        .map(|item| item.variant_id)
        .collect()
}
