//! Excel parser for the import wizard.
//!
//! Reads an `import.xlsx` file with three sheets:
//! - **Products** — one row per product
//! - **Variants** — one row per variant (linked by `product_handle`)
//! - **Classes** — optional; product type/category mapping
//!
//! All three sheets must have a header row as the first row.

use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

// ─── Public output types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRow {
    /// Row number in the spreadsheet (1-based, header = 1).
    pub row: usize,
    /// Fields parsed from the row, keyed by column header name.
    pub fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetData {
    pub sheet: String,
    pub headers: Vec<String>,
    pub rows: Vec<ImportRow>,
}

// ─── Public parser ────────────────────────────────────────────────────────────

/// Parse all sheets from an XLSX byte slice.
///
/// Returns a map of `sheet_name → SheetData`.
pub fn parse_xlsx(bytes: &[u8]) -> Result<std::collections::HashMap<String, SheetData>, String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut wb: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| format!("Cannot open workbook: {e}"))?;

    let sheet_names: Vec<String> = wb.sheet_names().to_vec();
    let mut result = std::collections::HashMap::new();

    for sheet_name in sheet_names {
        if let Ok(range) = wb.worksheet_range(&sheet_name) {
            let mut rows_iter = range.rows();
            let headers: Vec<String> = match rows_iter.next() {
                Some(h) => h.iter().map(|c| cell_string_val(c)).collect(),
                None => continue,
            };

            let mut rows = Vec::new();
            for (i, row) in rows_iter.enumerate() {
                let mut fields = std::collections::HashMap::new();
                for (j, header) in headers.iter().enumerate() {
                    if let Some(cell) = row.get(j) {
                        let val = cell_string_val(cell);
                        if !val.is_empty() {
                            fields.insert(header.clone(), val);
                        }
                    }
                }
                if !fields.is_empty() {
                    rows.push(ImportRow { row: i + 2, fields });
                }
            }

            result.insert(
                sheet_name.clone(),
                SheetData {
                    sheet: sheet_name,
                    headers,
                    rows,
                },
            );
        }
    }

    Ok(result)
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn cell_string_val(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Empty => String::new(),
        Data::Error(e) => format!("#ERR:{e:?}"),
    }
}
