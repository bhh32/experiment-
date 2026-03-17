//! # rw-mail-merge
//!
//! Mail merge functionality for Rust Writer.
//!
//! - Data source connection (CSV, JSON, spreadsheets)
//! - Field mapping (data columns to merge fields)
//! - Preview merged results
//! - Output to individual documents, printer, or email
//! - Rules (conditional fields: IF, SKIP, NEXT)
//! - Address block and greeting line helpers

use std::collections::HashMap;

/// A data source for mail merge.
#[derive(Debug, Clone)]
pub struct DataSource {
    /// Column names
    pub columns: Vec<String>,
    /// Rows of data (each row is a map of column name to value)
    pub records: Vec<HashMap<String, String>>,
}

/// Data source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSourceType {
    Csv,
    Json,
    Manual,
}

/// Mail merge output target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeOutput {
    /// Create individual documents
    NewDocuments,
    /// Send to printer
    Printer,
    /// Send as email
    Email,
}

/// Mail merge configuration.
#[derive(Debug, Clone)]
pub struct MergeConfig {
    pub source_type: DataSourceType,
    pub output: MergeOutput,
    /// Which records to merge (None = all)
    pub record_range: Option<(usize, usize)>,
}
