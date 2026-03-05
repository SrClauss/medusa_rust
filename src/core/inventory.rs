//! Inventory management — stock levels and reservations.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Inventory level at a specific location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLevel {
    pub inventory_item_id: Uuid,
    pub location_id: Uuid,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
}

impl InventoryLevel {
    /// Returns the quantity available for new orders.
    pub fn available_quantity(&self) -> i32 {
        self.stocked_quantity - self.reserved_quantity
    }
}

/// Reserves `quantity` units for a specific variant / location.
///
/// Returns an error if there is insufficient available stock.
pub async fn reserve_inventory(
    pool: &PgPool,
    inventory_item_id: Uuid,
    location_id: Uuid,
    quantity: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let level = sqlx::query!(
        r#"SELECT stocked_quantity, reserved_quantity
           FROM inventory_levels
           WHERE inventory_item_id = $1 AND location_id = $2
           FOR UPDATE"#,
        inventory_item_id,
        location_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Inventory level not found".into()))?;

    let available = level.stocked_quantity - level.reserved_quantity;
    if available < quantity {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock: requested {quantity}, available {available}"
        )));
    }

    sqlx::query!(
        r#"UPDATE inventory_levels
           SET reserved_quantity = reserved_quantity + $1
           WHERE inventory_item_id = $2 AND location_id = $3"#,
        quantity,
        inventory_item_id,
        location_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Releases a previously made reservation.
pub async fn release_reservation(
    pool: &PgPool,
    inventory_item_id: Uuid,
    location_id: Uuid,
    quantity: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"UPDATE inventory_levels
           SET reserved_quantity = GREATEST(0, reserved_quantity - $1)
           WHERE inventory_item_id = $2 AND location_id = $3"#,
        quantity,
        inventory_item_id,
        location_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
