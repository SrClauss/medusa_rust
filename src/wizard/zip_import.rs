//! Zip-based import pipeline.
//!
//! Accepts a multipart upload of a `.zip` file.  The zip must contain:
//! - `import.xlsx`            — the product data workbook
//! - `assets/<slug>/…`       — product images (optional)
//!
//! The import is dispatched as an Apalis background job backed by PostgreSQL so
//! that the HTTP request can return immediately (job ID) and the caller can
//! poll `/wizard/import/:job_id/status` for progress.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    error::AppError,
    wizard::{excel, slugify},
};

// ─── Job payload ──────────────────────────────────────────────────────────────

/// Represents the persisted background-job payload.
///
/// This struct is serialised to JSON and stored in the `apalis_jobs` table.
/// Apalis workers pick it up, deserialise it, and call `run_import_job`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub job_id: Uuid,
    /// Raw bytes of the `.zip` file encoded as base64 for JSON storage.
    pub zip_base64: String,
    /// ID of the admin user who triggered the import.
    pub triggered_by: Uuid,
}

// ─── Job status ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ─── Core import logic ────────────────────────────────────────────────────────

/// Executes the full import pipeline inside a single SQLx transaction.
///
/// On success every class, product, variant and asset mapping is persisted
/// atomically.  On any error the transaction is rolled back and a detailed
/// error listing (sheet + row + message) is returned to the caller.
pub async fn run_import_job(job: &ImportJob, pool: &PgPool) -> Result<(), AppError> {
    tracing::info!(job_id = %job.job_id, "Starting import job");

    // 1. Decode zip bytes.
    let zip_bytes = base64_decode(&job.zip_base64)?;

    // 2. Extract the workbook and asset index from the zip.
    let (xlsx_bytes, asset_map) = extract_zip(&zip_bytes)?;

    // 3. Parse and validate the Excel workbook (all errors reported at once).
    let sheet_data = excel::parse_excel(&xlsx_bytes).map_err(|validation_errors| {
        let details = validation_errors
            .iter()
            .map(|e| {
                format!(
                    "[{}] Row {}: {}",
                    e.sheet,
                    e.row,
                    e.errors.join("; ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!(job_id = %job.job_id, "Validation errors:\n{}", details);
        AppError::Validation(details)
    })?;

    // 4. Persist inside an atomic SQLx transaction.
    let mut tx = pool.begin().await?;

    // 4a. Upsert product categories (classes).
    let mut class_id_map: HashMap<String, Uuid> = HashMap::new();
    for class in &sheet_data.classes {
        let handle = class
            .handle
            .clone()
            .unwrap_or_else(|| slugify(&class.name));
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO product_categories (id, name, handle, description, created_at, updated_at)
               VALUES ($1, $2, $3, $4, NOW(), NOW())
               ON CONFLICT (handle) DO UPDATE
               SET name = EXCLUDED.name, description = EXCLUDED.description, updated_at = NOW()
               RETURNING id"#,
            id,
            class.name,
            handle,
            class.description,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to upsert class '{}': {}", class.name, e))
        })?;
        class_id_map.insert(class.name.clone(), id);
    }

    // 4b. Upsert products.
    let mut product_id_map: HashMap<String, Uuid> = HashMap::new();
    for product in &sheet_data.products {
        let handle = product
            .handle
            .clone()
            .unwrap_or_else(|| slugify(&product.title));
        let id = Uuid::new_v4();
        let category_id = product
            .class
            .as_ref()
            .and_then(|c| class_id_map.get(c))
            .copied();
        let status = product.status.as_deref().unwrap_or("draft");

        sqlx::query!(
            r#"INSERT INTO products (id, title, handle, description, status, category_id, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
               ON CONFLICT (handle) DO UPDATE
               SET title = EXCLUDED.title, description = EXCLUDED.description,
                   status = EXCLUDED.status, updated_at = NOW()
               RETURNING id"#,
            id,
            product.title,
            handle,
            product.description,
            status,
            category_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to upsert product '{}': {}", product.title, e))
        })?;
        product_id_map.insert(product.title.clone(), id);
    }

    // 4c. Upsert variants and link assets.
    for variant in &sheet_data.variants {
        let product_id = product_id_map
            .get(&variant.product_title)
            .copied()
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "Variant SKU '{}' references unknown product '{}'",
                    variant.sku, variant.product_title
                ))
            })?;

        let currency_code = variant
            .currency_code
            .as_deref()
            .unwrap_or("brl")
            .to_lowercase();

        // Convert decimal price to cents.
        let price_cents = (variant.price * 100.0).round() as i64;

        let variant_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO product_variants (id, product_id, sku, price, currency_code, inventory_quantity, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
               ON CONFLICT (sku) DO UPDATE
               SET price = EXCLUDED.price, inventory_quantity = EXCLUDED.inventory_quantity, updated_at = NOW()
               RETURNING id"#,
            variant_id,
            product_id,
            variant.sku,
            price_cents,
            currency_code,
            variant.stock,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to upsert variant SKU '{}': {}", variant.sku, e))
        })?;

        // Link assets: look for images in assets/<slug>/ or assets/<sku>/
        let slug_path = slugify(&variant.product_title);
        let sku_path = slugify(&variant.sku);

        for (asset_path, asset_bytes) in asset_map.iter() {
            let asset_lower = asset_path.to_lowercase();
            if asset_lower.contains(&slug_path) || asset_lower.contains(&sku_path) {
                // Persist asset metadata; actual bytes would be written to blob
                // storage in a production implementation.
                let asset_id = Uuid::new_v4();
                let filename = Path::new(asset_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(asset_path.as_str());

                sqlx::query!(
                    r#"INSERT INTO product_images (id, variant_id, filename, size_bytes, created_at)
                       VALUES ($1, $2, $3, $4, NOW())
                       ON CONFLICT DO NOTHING"#,
                    asset_id,
                    variant_id,
                    filename,
                    asset_bytes.len() as i64,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    AppError::Internal(format!(
                        "Failed to insert image '{}' for SKU '{}': {}",
                        filename, variant.sku, e
                    ))
                })?;
            }
        }
    }

    tx.commit().await?;
    tracing::info!(job_id = %job.job_id, "Import job completed successfully");
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extracts `import.xlsx` bytes and builds a map of `asset_path → bytes`
/// from the zip archive.
fn extract_zip(zip_bytes: &[u8]) -> Result<(Vec<u8>, HashMap<String, Vec<u8>>), AppError> {
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::BadRequest(format!("Invalid zip file: {e}")))?;

    let mut xlsx_bytes: Option<Vec<u8>> = None;
    let mut asset_map: HashMap<String, Vec<u8>> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Internal(format!("Zip read error: {e}")))?;

        let name = file.name().to_string();

        if name.ends_with("import.xlsx") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| AppError::Internal(format!("Failed to read import.xlsx: {e}")))?;
            xlsx_bytes = Some(buf);
        } else if name.starts_with("assets/") && !name.ends_with('/') {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| AppError::Internal(format!("Failed to read asset {name}: {e}")))?;
            // Store the path relative to the zip root.
            asset_map.insert(name, buf);
        }
    }

    let xlsx = xlsx_bytes
        .ok_or_else(|| AppError::BadRequest("import.xlsx not found in zip".into()))?;
    Ok((xlsx, asset_map))
}

/// Decodes a base64-encoded byte vector.
fn base64_decode(encoded: &str) -> Result<Vec<u8>, AppError> {
    use std::io::Read;
    // Use the standard alphabet with padding.
    let mut decoder = base64_decoder(encoded.as_bytes());
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| AppError::BadRequest(format!("Invalid base64 payload: {e}")))?;
    Ok(out)
}

/// Minimal base64 decoder (avoids adding a heavy dependency).
fn base64_decoder(input: &[u8]) -> impl Read + '_ {
    Base64Decoder::new(input)
}

struct Base64Decoder<'a> {
    input: &'a [u8],
    buf: Vec<u8>,
    pos: usize,
    done: bool,
}

impl<'a> Base64Decoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        let decoded = decode_base64_bytes(input);
        Self { input, buf: decoded, pos: 0, done: false }
    }
}

impl<'a> Read for Base64Decoder<'a> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.buf.len() {
            return Ok(0);
        }
        let n = out.len().min(self.buf.len() - self.pos);
        out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

fn decode_base64_bytes(input: &[u8]) -> Vec<u8> {
    // Simple base64 alphabet table.
    const TABLE: [i8; 256] = {
        let mut t = [-1i8; 256];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0usize;
        while i < chars.len() {
            t[chars[i] as usize] = i as i8;
            i += 1;
        }
        t
    };

    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 4);
    let mut buf = [0u8; 4];
    let mut buf_len = 0;

    for &byte in input {
        if byte == b'=' || byte == b'\n' || byte == b'\r' || byte == b' ' {
            continue;
        }
        let val = TABLE[byte as usize];
        if val < 0 {
            continue;
        }
        buf[buf_len] = val as u8;
        buf_len += 1;
        if buf_len == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
            out.push((buf[2] << 6) | buf[3]);
            buf_len = 0;
        }
    }
    match buf_len {
        2 => out.push((buf[0] << 2) | (buf[1] >> 4)),
        3 => {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
        }
        _ => {}
    }
    out
}

/// Encodes raw bytes as base64 (used when queuing jobs).
pub fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() * 4 + 2) / 3);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
    }
    out
}
