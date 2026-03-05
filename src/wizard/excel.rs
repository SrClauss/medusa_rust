//! Excel parser — reads `import.xlsx` using the `calamine` crate.
//!
//! Expected sheets:
//! - **Classes**  — product category / type definitions
//! - **Products** — product master data
//! - **Variants** — variant SKUs, prices and stock

use calamine::{open_workbook_from_rs, DataType, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use validator::Validate;

use crate::error::AppError;

// ─── Row models ───────────────────────────────────────────────────────────────

/// A row from the *Classes* sheet (product categories / types).
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ClassRow {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
    pub handle: Option<String>,
}

/// A row from the *Products* sheet.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProductRow {
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub class: Option<String>,
    pub handle: Option<String>,
    pub status: Option<String>,
}

/// A row from the *Variants* sheet.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VariantRow {
    #[validate(length(min = 1, message = "product_title is required"))]
    pub product_title: String,
    #[validate(length(min = 1, message = "sku is required"))]
    pub sku: String,
    #[validate(range(min = 0.0, message = "price must be non-negative"))]
    pub price: f64,
    pub currency_code: Option<String>,
    #[validate(range(min = 0, message = "stock must be non-negative"))]
    pub stock: i32,
}

/// Aggregated data parsed from all three sheets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetData {
    pub classes: Vec<ClassRow>,
    pub products: Vec<ProductRow>,
    pub variants: Vec<VariantRow>,
}

/// A validation error tied to a specific sheet and row number (1-based).
#[derive(Debug, Serialize)]
pub struct ImportRow {
    pub sheet: String,
    pub row: usize,
    pub errors: Vec<String>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

/// Parses an Excel workbook from raw bytes.
///
/// Returns `SheetData` on success, or a list of `ImportRow` errors describing
/// every invalid cell along with its sheet name and 1-based row index.
pub fn parse_excel(bytes: &[u8]) -> Result<SheetData, Vec<ImportRow>> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> =
        open_workbook_from_rs(cursor).map_err(|e| {
            vec![ImportRow {
                sheet: "workbook".into(),
                row: 0,
                errors: vec![format!("Failed to open workbook: {e}")],
            }]
        })?;

    let classes = parse_classes(&mut workbook)?;
    let products = parse_products(&mut workbook)?;
    let variants = parse_variants(&mut workbook)?;

    Ok(SheetData { classes, products, variants })
}

// ─── Sheet parsers ────────────────────────────────────────────────────────────

fn parse_classes<R: std::io::Read + std::io::Seek>(
    wb: &mut Xlsx<R>,
) -> Result<Vec<ClassRow>, Vec<ImportRow>> {
    let sheet = wb
        .worksheet_range("Classes")
        .map_err(|e| vec![import_err("Classes", 0, &e.to_string())])?;

    let mut rows = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, row) in sheet.rows().enumerate().skip(1) {
        let name = cell_string(row, 0);
        let description = cell_opt_string(row, 1);
        let handle = cell_opt_string(row, 2);

        let record = ClassRow { name, description, handle };
        if let Err(e) = record.validate() {
            errors.push(ImportRow {
                sheet: "Classes".into(),
                row: row_idx + 1,
                errors: validation_messages(&e),
            });
        } else {
            rows.push(record);
        }
    }

    if errors.is_empty() { Ok(rows) } else { Err(errors) }
}

fn parse_products<R: std::io::Read + std::io::Seek>(
    wb: &mut Xlsx<R>,
) -> Result<Vec<ProductRow>, Vec<ImportRow>> {
    let sheet = wb
        .worksheet_range("Products")
        .map_err(|e| vec![import_err("Products", 0, &e.to_string())])?;

    let mut rows = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, row) in sheet.rows().enumerate().skip(1) {
        let title = cell_string(row, 0);
        let description = cell_opt_string(row, 1);
        let class = cell_opt_string(row, 2);
        let handle = cell_opt_string(row, 3);
        let status = cell_opt_string(row, 4);

        let record = ProductRow { title, description, class, handle, status };
        if let Err(e) = record.validate() {
            errors.push(ImportRow {
                sheet: "Products".into(),
                row: row_idx + 1,
                errors: validation_messages(&e),
            });
        } else {
            rows.push(record);
        }
    }

    if errors.is_empty() { Ok(rows) } else { Err(errors) }
}

fn parse_variants<R: std::io::Read + std::io::Seek>(
    wb: &mut Xlsx<R>,
) -> Result<Vec<VariantRow>, Vec<ImportRow>> {
    let sheet = wb
        .worksheet_range("Variants")
        .map_err(|e| vec![import_err("Variants", 0, &e.to_string())])?;

    let mut rows = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, row) in sheet.rows().enumerate().skip(1) {
        let product_title = cell_string(row, 0);
        let sku = cell_string(row, 1);
        let price = cell_f64(row, 2).unwrap_or(0.0);
        let currency_code = cell_opt_string(row, 3);
        let stock = cell_i32(row, 4).unwrap_or(0);

        let record = VariantRow { product_title, sku, price, currency_code, stock };
        if let Err(e) = record.validate() {
            errors.push(ImportRow {
                sheet: "Variants".into(),
                row: row_idx + 1,
                errors: validation_messages(&e),
            });
        } else {
            rows.push(record);
        }
    }

    if errors.is_empty() { Ok(rows) } else { Err(errors) }
}

// ─── Cell helpers ─────────────────────────────────────────────────────────────

fn cell_string(row: &[DataType], idx: usize) -> String {
    row.get(idx)
        .map(|c| match c {
            DataType::String(s) => s.trim().to_string(),
            DataType::Float(f) => f.to_string(),
            DataType::Int(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
            DataType::Empty => String::new(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

fn cell_opt_string(row: &[DataType], idx: usize) -> Option<String> {
    let s = cell_string(row, idx);
    if s.is_empty() { None } else { Some(s) }
}

fn cell_f64(row: &[DataType], idx: usize) -> Option<f64> {
    row.get(idx).and_then(|c| match c {
        DataType::Float(f) => Some(*f),
        DataType::Int(i) => Some(*i as f64),
        DataType::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

fn cell_i32(row: &[DataType], idx: usize) -> Option<i32> {
    row.get(idx).and_then(|c| match c {
        DataType::Int(i) => Some(*i as i32),
        DataType::Float(f) => Some(*f as i32),
        DataType::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

fn import_err(sheet: &str, row: usize, msg: &str) -> ImportRow {
    ImportRow {
        sheet: sheet.to_string(),
        row,
        errors: vec![msg.to_string()],
    }
}

fn validation_messages(e: &validator::ValidationErrors) -> Vec<String> {
    e.field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(|ve| {
                format!(
                    "Field '{}': {}",
                    field,
                    ve.message.clone().unwrap_or_default()
                )
            })
        })
        .collect()
}
