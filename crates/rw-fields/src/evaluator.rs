//! Field evaluation engine.

use crate::FieldValue;

/// Context provided to field evaluation.
pub struct FieldContext {
    /// Current page number
    pub current_page: usize,
    /// Total page count
    pub total_pages: usize,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document filename
    pub filename: Option<String>,
    /// Current date/time
    pub now: chrono::DateTime<chrono::Utc>,
}

/// Evaluate a date/time format string.
pub fn format_datetime(dt: &chrono::DateTime<chrono::Utc>, format: &str) -> FieldValue {
    FieldValue::Text(dt.format(format).to_string())
}
