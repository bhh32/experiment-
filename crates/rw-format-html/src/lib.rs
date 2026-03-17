//! # rw-format-html
//!
//! HTML import/export for Rust Writer.
//!
//! - Import HTML files, mapping HTML elements to document model
//! - Export documents as clean, semantic HTML5
//! - CSS-based styling in exported HTML
//! - Inline images as base64 data URIs or external references
//! - Paste from HTML clipboard content

pub mod reader;
pub mod writer;

use rw_document::Document;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HtmlError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Read an HTML document from a file path.
pub fn read_html(path: &std::path::Path) -> Result<Document, HtmlError> {
    reader::read_html(path)
}

/// Parse an HTML string into a Document (useful for EPUB import etc.).
pub fn parse_html(html: &str) -> Result<Document, HtmlError> {
    reader::parse_html(html)
}

/// Write a document to HTML format at the given path.
pub fn write_html(doc: &Document, path: &std::path::Path) -> Result<(), HtmlError> {
    writer::write_html(doc, path)
}

/// Generate an HTML string from a document (without writing to disk).
pub fn generate_html(doc: &Document) -> String {
    writer::generate_html(doc)
}
