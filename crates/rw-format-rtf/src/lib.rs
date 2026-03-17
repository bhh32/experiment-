//! # rw-format-rtf
//!
//! RTF (Rich Text Format) reader/writer for Rust Writer.
//!
//! RTF is a plain-text markup format using control words prefixed with `\`.
//! Example: `{\rtf1\ansi{\b Hello} World}`
//!
//! This is a custom implementation as no mature RTF crate exists in the
//! Rust ecosystem. We support RTF 1.9.1 specification.

pub mod reader;
pub mod writer;

use rw_document::Document;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RtfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error at position {position}: {message}")]
    Parse { position: usize, message: String },
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

/// Read an RTF document from a file path.
pub fn read_rtf(path: &std::path::Path) -> Result<Document, RtfError> {
    reader::read_rtf(path)
}

/// Write a document to RTF format at the given path.
pub fn write_rtf(doc: &Document, path: &std::path::Path) -> Result<(), RtfError> {
    writer::write_rtf(doc, path)
}
