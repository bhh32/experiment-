//! # rw-format-epub
//!
//! EPUB export for Rust Writer.
//!
//! Generates EPUB 3.0 ebooks from documents. EPUB is essentially
//! a ZIP archive containing XHTML content, CSS styling, and metadata.
//!
//! - Chapter splitting (by heading level)
//! - Cover image
//! - Table of contents (NCX + HTML TOC)
//! - Embedded images
//! - Metadata (title, author, language, ISBN)
//! - CSS styling

use rw_document::Document;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("EPUB generation error: {0}")]
    Generation(String),
}

/// EPUB export options.
#[derive(Debug, Clone)]
pub struct EpubOptions {
    /// Split chapters at this heading level (1 = Heading 1)
    pub split_level: u8,
    /// Cover image path
    pub cover_image: Option<String>,
    /// CSS stylesheet content
    pub custom_css: Option<String>,
    /// EPUB version
    pub version: EpubVersion,
}

impl Default for EpubOptions {
    fn default() -> Self {
        Self {
            split_level: 1,
            cover_image: None,
            custom_css: None,
            version: EpubVersion::V3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpubVersion {
    V2,
    V3,
}

/// Export a document as EPUB.
pub fn write_epub(
    doc: &Document,
    path: &std::path::Path,
    options: &EpubOptions,
) -> Result<(), EpubError> {
    let _ = (doc, path, options);
    // TODO: Implement EPUB export
    Ok(())
}
