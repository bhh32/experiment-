//! # rw-fields
//!
//! The fields system for Rust Writer.
//!
//! Fields are dynamic content elements that display computed values:
//! - Page number / page count
//! - Date / time (with format strings)
//! - Document properties (title, author, filename)
//! - Cross-references to bookmarks, headings, figures
//! - Sequence numbers (for figure/table numbering)
//! - Table of contents generation
//! - Index generation
//! - Bibliography generation
//! - Conditional fields (IF)
//! - Input fields (user prompts)
//! - Mail merge fields
//!
//! Fields are stored in the document model as `FieldRef` inline
//! elements and are updated (re-evaluated) on demand.

pub mod bookmark;
pub mod cross_ref;
pub mod evaluator;
pub mod toc;

/// A bookmark — a named location in the document.
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub name: String,
    /// The text content that the bookmark spans (for cross-reference display)
    pub display_text: Option<String>,
}

/// The result of evaluating a field.
#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Error(String),
}

impl FieldValue {
    pub fn as_display_string(&self) -> String {
        match self {
            FieldValue::Text(s) => s.clone(),
            FieldValue::Number(n) => n.to_string(),
            FieldValue::Error(e) => format!("Error: {}", e),
        }
    }
}
