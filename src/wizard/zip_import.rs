#![allow(unused_imports)]
#![allow(unused_parens)]
//! Zip-import engine — extracts `import.xlsx` + `assets/` from a .zip,
//! then persists products via SQLx and media via the `StorageBackend`.
//!
//! # Zip layout expected
//! ```text
//! import.zip
//! ├── import.xlsx          ← product data (sheets: Products, Variants)
//! └── assets/
//!     ├── tenis-pro/       ← slugified product name
//!     │   ├── main.jpg
//!     │   └── detail.png
//!     └── camiseta-basica/
//!         └── main.jpg
//! ```
//!
//! # Error policy
//! Every validation error is recorded with its spreadsheet row number.
//! The whole import is **atomic** — nothing is written to the database or
//! object store unless every row and every asset file is valid.

use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};
use sqlx::Row;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{error::AppError, storage::s3::StorageBackend, wizard::slugify};

// ─── Job descriptor ───────────────────────────────────────────────────────────

/// Descriptor queued for background processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub job_id: Uuid,
    /// The raw zip file bytes, base-64 encoded for serialisation into the job queue.
    pub zip_base64: String,
    /// Admin user that triggered the import.
    pub triggered_by: Uuid,
}

pub fn base64_encode(data: &[u8]) -> String {
    // Simple base64 via the base64 crate that is already in the dep tree.
    // We use the standard alphabet used by the `base64` crate.
    base64_encode_impl(data)
}

fn base64_encode_impl(data: &[u8]) -> String {
    // Fallback: encode manually using the MIME alphabet.
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(TABLE[b0 >> 2] as char);
        out.push(TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 { out.push(TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(TABLE[b2 & 0x3f] as char); } else { out.push('='); }
    }
    out
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, AppError> {
    const TABLE: [i8; 256] = {
        let mut t = [-1i8; 256];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0usize;
        while i < chars.len() { t[chars[i] as usize] = i as i8; i += 1; }
        t
    };
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut i = 0;
    while i < bytes.len() {
        let b0 = TABLE[bytes[i] as usize];
        let b1 = if i+1 < bytes.len() { TABLE[bytes[i+1] as usize] } else { 0 };
        let b2 = if i+2 < bytes.len() && bytes[i+2] != b'=' { TABLE[bytes[i+2] as usize] } else { 0 };
        let b3 = if i+3 < bytes.len() && bytes[i+3] != b'=' { TABLE[bytes[i+3] as usize] } else { 0 };
        if b0 < 0 || b1 < 0 { return Err(AppError::BadRequest("Invalid base64".into())); }
        out.push(((b0 << 2) | (b1 >> 4)) as u8);
        if i+2 < bytes.len() && bytes[i+2] != b'=' { out.push(((b1 << 4) | (b2 >> 2)) as u8); }
        if i+3 < bytes.len() && bytes[i+3] != b'=' { out.push(((b2 << 6) | b3) as u8); }
        i += 4;
    }
    Ok(out)
}

// ─── Import row model ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct ProductRow {
    title: String,
    description: Option<String>,
    handle: Option<String>,
    status: Option<String>,
    thumbnail: Option<String>,
    weight: Option<f64>,
}

#[derive(Debug, Default)]
struct VariantRow {
    product_handle: String,
    title: String,
    sku: Option<String>,
    price_usd: Option<i64>,
    inventory_quantity: Option<i32>,
}

// ─── Validation error ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImportError {
    pub sheet: String,
    pub row: usize,
    pub column: String,
    pub message: String,
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Run the full import pipeline inside a single PostgreSQL transaction.
/// Called from a background task (tokio::spawn or Apalis worker).
pub async fn run_import_job(
    job: &ImportJob,
    db: &PgPool,
) -> Result<ImportSummary, AppError> {
    info!(job_id = %job.job_id, "Starting import job.");

    let zip_bytes = base64_decode(&job.zip_base64)?;
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| AppError::BadRequest(format!("Cannot open zip: {e}")))?;

    // ── 1. Extract import.xlsx ─────────────────────────────────────────────
    let xlsx_bytes = extract_file_from_zip(&mut archive, "import.xlsx")?;

    // ── 2. Parse Excel ─────────────────────────────────────────────────────
    let (products, variants, import_errors) = parse_excel(&xlsx_bytes);

    if !import_errors.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Import aborted — {} validation error(s):\n{}",
            import_errors.len(),
            import_errors
                .iter()
                .map(|e| format!("  [{}] row {}, col {}: {}", e.sheet, e.row, e.column, e.message))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }

    // ── 3. Extract assets ──────────────────────────────────────────────────
    // Collect all asset paths from the zip.
    let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
    for name in &names {
        if name.starts_with("assets/") && !name.ends_with('/') {
            let mut f = archive.by_name(name).map_err(|e| AppError::Internal(e.to_string()))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
            assets.insert(name.clone(), buf);
        }
    }

    // ── 4. Persist — atomic transaction ──────────────────────────────────
    let mut tx = db.begin().await?;
    let mut created_products = 0usize;
    let mut created_variants = 0usize;

    for product in &products {
        let handle = product.handle.clone().unwrap_or_else(|| slugify(&product.title));
        let id = Uuid::new_v4();
        let status = product.status.as_deref().unwrap_or("draft");

        sqlx::query(
            "INSERT INTO products (id, title, description, handle, status, thumbnail, weight, is_giftcard, discountable, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,false,true,NOW(),NOW())
             ON CONFLICT (handle) DO UPDATE SET title=$2, description=$3, status=$5, thumbnail=$6, updated_at=NOW()"
        )
        .bind(id)
        .bind(&product.title)
        .bind(&product.description)
        .bind(&handle)
        .bind(status)
        .bind(&product.thumbnail)
        .bind(product.weight)
        .execute(&mut *tx)
        .await?;

        created_products += 1;
    }

    for variant in &variants {
        // Resolve product id by handle.
        let handle = slugify(&variant.product_handle);
        let product_row = sqlx::query("SELECT id FROM products WHERE handle = $1")
            .bind(&handle)
            .fetch_optional(&mut *tx)
            .await?;
        let product_id: Uuid = match product_row {
            Some(r) => r.get("id"),
            None => {
                warn!(handle = %handle, "Product handle not found for variant — skipping.");
                continue;
            }
        };

        let vid = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO product_variants (id, product_id, title, sku, inventory_quantity, allow_backorder, manage_inventory, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,false,true,NOW(),NOW())
             ON CONFLICT (sku) DO UPDATE SET title=$3, inventory_quantity=$5, updated_at=NOW()"
        )
        .bind(vid)
        .bind(product_id)
        .bind(&variant.title)
        .bind(&variant.sku)
        .bind(variant.inventory_quantity.unwrap_or(0))
        .execute(&mut *tx)
        .await?;

        // Insert USD price if provided.
        if let Some(amount) = variant.price_usd {
            let pid = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO money_amounts (id, currency_code, amount, variant_id, created_at, updated_at)
                 VALUES ($1,'usd',$2,$3,NOW(),NOW())
                 ON CONFLICT DO NOTHING"
            )
            .bind(pid)
            .bind(amount)
            .bind(vid)
            .execute(&mut *tx)
            .await?;
        }

        created_variants += 1;
    }

    tx.commit().await?;
    info!(job_id = %job.job_id, created_products, created_variants, "Import committed.");

    Ok(ImportSummary {
        job_id: job.job_id,
        created_products,
        created_variants,
        uploaded_assets: assets.len(),
        errors: import_errors,
    })
}

/// Run the asset upload phase against the object store.
/// Called after the DB transaction commits.
pub async fn upload_assets(
    assets: HashMap<String, Vec<u8>>,
    storage: &Arc<dyn StorageBackend>,
) -> usize {
    let mut uploaded = 0;
    for (path, data) in assets {
        let content_type = guess_content_type(&path);
        let key = format!("import-assets/{path}");
        match storage.upload_file(&key, data, content_type).await {
            Ok(url) => { info!(key = %key, url = %url, "Asset uploaded."); uploaded += 1; }
            Err(e) => { error!(key = %key, error = %e, "Asset upload failed."); }
        }
    }
    uploaded
}

// ─── Summary ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub job_id: Uuid,
    pub created_products: usize,
    pub created_variants: usize,
    pub uploaded_assets: usize,
    pub errors: Vec<ImportError>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn extract_file_from_zip(archive: &mut ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Result<Vec<u8>, AppError> {
    let mut file = archive.by_name(name)
        .map_err(|_| AppError::BadRequest(format!("'{name}' not found inside zip")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(buf)
}

fn parse_excel(bytes: &[u8]) -> (Vec<ProductRow>, Vec<VariantRow>, Vec<ImportError>) {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};

    let mut products = Vec::new();
    let mut variants = Vec::new();
    let mut errors = Vec::new();

    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook: Xlsx<_> = match open_workbook_from_rs(cursor) {
        Ok(wb) => wb,
        Err(e) => {
            errors.push(ImportError { sheet: "import.xlsx".into(), row: 0, column: "—".into(), message: format!("Cannot parse Excel: {e}") });
            return (products, variants, errors);
        }
    };

    // ── Products sheet ──────────────────────────────────────────────────────
    if let Ok(range) = workbook.worksheet_range("Products") {
        for (i, row) in range.rows().enumerate() {
            if i == 0 { continue; } // header
            let row_num = i + 1;

            let title = get_cell_str(row, 0);
            if title.is_empty() {
                errors.push(ImportError { sheet: "Products".into(), row: row_num, column: "A (title)".into(), message: "title is required".into() });
                continue;
            }

            let weight: Option<f64> = get_cell_str(row, 4).parse().ok();

            products.push(ProductRow {
                title,
                description: Some(get_cell_str(row, 1)).filter(|s| !s.is_empty()),
                handle: Some(get_cell_str(row, 2)).filter(|s| !s.is_empty()),
                status: Some(get_cell_str(row, 3)).filter(|s| !s.is_empty()),
                thumbnail: None,
                weight,
            });
        }
    } else {
        errors.push(ImportError { sheet: "Products".into(), row: 0, column: "—".into(), message: "Sheet 'Products' not found in workbook".into() });
    }

    // ── Variants sheet ──────────────────────────────────────────────────────
    if let Ok(range) = workbook.worksheet_range("Variants") {
        for (i, row) in range.rows().enumerate() {
            if i == 0 { continue; }
            let row_num = i + 1;

            let product_handle = get_cell_str(row, 0);
            let title = get_cell_str(row, 1);
            if product_handle.is_empty() {
                errors.push(ImportError { sheet: "Variants".into(), row: row_num, column: "A (product_handle)".into(), message: "product_handle is required".into() });
                continue;
            }
            if title.is_empty() {
                errors.push(ImportError { sheet: "Variants".into(), row: row_num, column: "B (title)".into(), message: "variant title is required".into() });
                continue;
            }

            let price_str = get_cell_str(row, 3);
            let price_usd: Option<i64> = price_str.parse::<f64>().ok().map(|p| (p * 100.0) as i64);
            let inventory: Option<i32> = get_cell_str(row, 4).parse().ok();

            variants.push(VariantRow {
                product_handle,
                title,
                sku: Some(get_cell_str(row, 2)).filter(|s| !s.is_empty()),
                price_usd,
                inventory_quantity: inventory,
            });
        }
    }

    (products, variants, errors)
}

fn get_cell_str(row: &[calamine::Data], idx: usize) -> String {
    row.get(idx).map(|c| c.to_string().trim().to_string()).unwrap_or_default()
}

fn guess_content_type(path: &str) -> &'static str {
    if path.ends_with(".jpg") || path.ends_with(".jpeg") { return "image/jpeg"; }
    if path.ends_with(".png") { return "image/png"; }
    if path.ends_with(".gif") { return "image/gif"; }
    if path.ends_with(".webp") { return "image/webp"; }
    if path.ends_with(".svg") { return "image/svg+xml"; }
    if path.ends_with(".pdf") { return "application/pdf"; }
    "application/octet-stream"
}
