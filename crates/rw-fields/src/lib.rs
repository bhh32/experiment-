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

use std::collections::HashMap;
use rw_document::inline::FieldType;
use rw_document::Document;

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

/// Central manager for all field types, bookmarks, and cross-references.
#[derive(Debug, Default)]
pub struct FieldManager {
    /// Registry of sequence counters (e.g. "Figure" -> 3)
    sequence_counters: HashMap<String, u32>,
    /// Cached field evaluations
    cache: HashMap<String, String>,
}

impl FieldManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate a `FieldType` and return its display string.
    /// Delegates to `evaluator::evaluate_field`.
    pub fn evaluate(&self, field: &FieldType, doc: &Document) -> String {
        evaluator::evaluate_field(field, doc)
    }

    /// Update (re-evaluate) all fields in a document and cache results.
    pub fn update_fields(&mut self, doc: &Document) {
        self.cache.clear();
        for section in &doc.sections {
            for block in &section.content {
                self.visit_block_fields(block, doc);
            }
        }
    }

    fn visit_block_fields(&mut self, block: &rw_document::Block, doc: &Document) {
        use rw_document::Block;
        match block {
            Block::Paragraph(para) => {
                for inline in &para.content {
                    use rw_document::Inline;
                    if let Inline::Field(field_ref) = inline {
                        let key = format!("{:?}", field_ref.field_type);
                        let value = self.evaluate(&field_ref.field_type, doc);
                        self.cache.insert(key, value);
                    }
                }
            }
            Block::Table(tbl) => {
                for cell in &tbl.cells {
                    for b in &cell.content {
                        self.visit_block_fields(b, doc);
                    }
                }
            }
            _ => {}
        }
    }

    /// Get the next sequence number for a named sequence (e.g. "Figure").
    pub fn next_sequence(&mut self, name: &str) -> u32 {
        let counter = self.sequence_counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Reset all sequence counters.
    pub fn reset_sequences(&mut self) {
        self.sequence_counters.clear();
    }
}
