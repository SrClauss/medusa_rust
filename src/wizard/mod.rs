//! Wizard — Hyper-Fast Store Launch importer.
//!
//! Accepts a `.zip` file containing:
//! - `import.xlsx` with sheets: **Classes**, **Products**, **Variants**
//! - `assets/<slug>/` folders with product images (matched by SKU or name)
//!
//! The import runs in a background Apalis job to avoid API timeouts.

pub mod excel;
pub mod slugify;
pub mod zip_import;

pub use slugify::slugify;
// pub use zip_import::run_import_job;      // unused by the rest of codebase
