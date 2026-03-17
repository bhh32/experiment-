//! # rw-format-odf
//!
//! ODF (Open Document Format) .odt reader/writer for Rust Writer.
//!
//! An .odt file is a ZIP archive containing:
//! - `content.xml` — the document body (text, tables, images, etc.)
//! - `styles.xml` — style definitions
//! - `meta.xml` — document metadata
//! - `settings.xml` — application settings
//! - `manifest.xml` — file manifest
//! - `Pictures/` — embedded images
//! - `Thumbnails/thumbnail.png` — document thumbnail
//!
//! The ODF namespace is `urn:oasis:names:tc:opendocument:xmlns:*`
//! with sub-namespaces for office, text, style, table, draw, etc.

pub mod reader;
pub mod writer;

use rw_document::Document;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Invalid ODF structure: {0}")]
    InvalidStructure(String),
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

/// Read an ODF document from a file path.
pub fn read_odt(path: &std::path::Path) -> Result<Document, OdfError> {
    reader::read_odf(path)
}

/// Write a document to ODF format at the given path.
pub fn write_odt(doc: &Document, path: &std::path::Path) -> Result<(), OdfError> {
    writer::write_odf(doc, path)
}
