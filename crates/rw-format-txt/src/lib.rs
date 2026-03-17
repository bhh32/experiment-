//! # rw-format-txt
//!
//! Plain text import/export for Rust Writer.
//!
//! - Import plain text files (with encoding detection)
//! - Export document content as plain text
//! - Line ending handling (LF, CRLF, CR)
//! - Character encoding support (UTF-8, Latin-1, etc.)

use rw_document::Document;
use rw_document::block::Block;
use rw_document::paragraph::Paragraph;
use rw_document::section::Section;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TxtError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// Read a plain text file into a Document.
pub fn read_txt(path: &std::path::Path) -> Result<Document, TxtError> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = Document::new();
    doc.sections.clear();

    let mut section = Section::new();
    for line in content.lines() {
        section.content.push(Block::Paragraph(Paragraph::with_text(line)));
    }
    if section.content.is_empty() {
        section.content.push(Block::Paragraph(Paragraph::new()));
    }
    doc.sections.push(section);
    Ok(doc)
}

/// Export a document as plain text.
pub fn write_txt(doc: &Document, path: &std::path::Path) -> Result<(), TxtError> {
    let text = doc.plain_text();
    std::fs::write(path, text)?;
    Ok(())
}

/// Line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style (LF)
    Lf,
    /// Windows-style (CRLF)
    CrLf,
    /// Classic Mac (CR)
    Cr,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}
