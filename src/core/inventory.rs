//! Inventory management — stock levels and reservations.
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InventoryLevel {
    pub inventory_item_id: Uuid,
    pub location_id: Uuid,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
}
impl InventoryLevel {
    pub fn available_quantity(&self) -> i32 { self.stocked_quantity - self.reserved_quantity }
}

pub async fn reserve_inventory(pool: &PgPool, inventory_item_id: Uuid, location_id: Uuid, quantity: i32) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query("SELECT stocked_quantity, reserved_quantity FROM inventory_levels WHERE inventory_item_id = $1 AND location_id = $2 FOR UPDATE")
        .bind(inventory_item_id).bind(location_id)
        .fetch_optional(&mut *tx).await?
        .ok_or_else(|| AppError::NotFound("Inventory level not found".into()))?;
    let stocked: i32 = row.get("stocked_quantity");
    let reserved: i32 = row.get("reserved_quantity");
    let available = stocked - reserved;
    if available < quantity {
        return Err(AppError::BadRequest(format!("Insufficient stock: requested {quantity}, available {available}")));
    }
    sqlx::query("UPDATE inventory_levels SET reserved_quantity = reserved_quantity + $1 WHERE inventory_item_id = $2 AND location_id = $3")
        .bind(quantity).bind(inventory_item_id).bind(location_id)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn release_reservation(pool: &PgPool, inventory_item_id: Uuid, location_id: Uuid, quantity: i32) -> Result<(), AppError> {
    sqlx::query("UPDATE inventory_levels SET reserved_quantity = GREATEST(0, reserved_quantity - $1) WHERE inventory_item_id = $2 AND location_id = $3")
        .bind(quantity).bind(inventory_item_id).bind(location_id)
        .execute(pool).await?;
    Ok(())
}
