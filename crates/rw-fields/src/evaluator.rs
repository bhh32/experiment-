//! Field evaluation engine.

use crate::FieldValue;
use rw_document::inline::FieldType;
use rw_document::Document;

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

impl Default for FieldContext {
    fn default() -> Self {
        Self {
            current_page: 1,
            total_pages: 1,
            title: None,
            author: None,
            filename: None,
            now: chrono::Utc::now(),
        }
    }
}

/// Evaluate a date/time format string.
pub fn format_datetime(dt: &chrono::DateTime<chrono::Utc>, format: &str) -> FieldValue {
    FieldValue::Text(dt.format(format).to_string())
}

/// Evaluate a `FieldType` against the document and return a display string.
///
/// Page-number-dependent fields return placeholder values since layout
/// information is not available at this layer.
pub fn evaluate_field(field: &FieldType, doc: &Document) -> String {
    match field {
        FieldType::PageNumber => "1".to_string(),
        FieldType::PageCount => "1".to_string(),
        FieldType::Date { format } => {
            let now = chrono::Utc::now();
            now.format(format).to_string()
        }
        FieldType::Time { format } => {
            let now = chrono::Utc::now();
            now.format(format).to_string()
        }
        FieldType::FileName => "Document".to_string(),
        FieldType::Author => doc
            .metadata
            .author
            .clone()
            .unwrap_or_default(),
        FieldType::Title => doc
            .metadata
            .title
            .clone()
            .unwrap_or_default(),
        FieldType::Subject => doc
            .metadata
            .subject
            .clone()
            .unwrap_or_default(),
        FieldType::Custom { name: _, value } => value.clone(),
        FieldType::CrossReference { bookmark_name, ref_type: _ } => {
            // Without layout we can only return the bookmark name as a hint
            format!("[{}]", bookmark_name)
        }
        FieldType::Sequence { name } => {
            // Sequence numbering requires stateful evaluation; return placeholder
            format!("[SEQ {}]", name)
        }
        FieldType::MergeField { field_name } => {
            format!("«{}»", field_name)
        }
        FieldType::If { condition, true_text, false_text } => {
            // Simple evaluation: if condition is non-empty, return true_text
            if !condition.trim().is_empty() {
                true_text.clone()
            } else {
                false_text.clone()
            }
        }
        FieldType::TableOfContents => "[Table of Contents]".to_string(),
        FieldType::Index => "[Index]".to_string(),
        FieldType::Bibliography => "[Bibliography]".to_string(),
    }
}

/// Evaluate a field with a provided context (for more accurate page numbers, etc.).
pub fn evaluate_field_with_context(field: &FieldType, ctx: &FieldContext) -> String {
    match field {
        FieldType::PageNumber => ctx.current_page.to_string(),
        FieldType::PageCount => ctx.total_pages.to_string(),
        FieldType::Date { format } => ctx.now.format(format).to_string(),
        FieldType::Time { format } => ctx.now.format(format).to_string(),
        FieldType::FileName => ctx
            .filename
            .clone()
            .unwrap_or_else(|| "Document".to_string()),
        FieldType::Author => ctx.author.clone().unwrap_or_default(),
        FieldType::Title => ctx.title.clone().unwrap_or_default(),
        FieldType::Subject => String::new(),
        FieldType::Custom { value, .. } => value.clone(),
        FieldType::CrossReference { bookmark_name, .. } => format!("[{}]", bookmark_name),
        FieldType::Sequence { name } => format!("[SEQ {}]", name),
        FieldType::MergeField { field_name } => format!("«{}»", field_name),
        FieldType::If { condition, true_text, false_text } => {
            if !condition.trim().is_empty() {
                true_text.clone()
            } else {
                false_text.clone()
            }
        }
        FieldType::TableOfContents => "[Table of Contents]".to_string(),
        FieldType::Index => "[Index]".to_string(),
        FieldType::Bibliography => "[Bibliography]".to_string(),
    }
}
