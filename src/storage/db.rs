//! Database helpers — connection pool construction.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

use crate::error::AppError;

/// Creates a PostgreSQL connection pool from the `DATABASE_URL` environment
/// variable, with sensible defaults for production workloads.
pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
        .map_err(AppError::from)?;

    tracing::info!("PostgreSQL connection pool established.");
    Ok(pool)
}

/// Runs all pending SQLx migrations from the `migrations/` directory.
pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    tracing::info!("Database migrations applied.");
    Ok(())
}
