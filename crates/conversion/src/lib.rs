mod markdown;
mod docx;
mod pandoc;

pub use markdown::{markdown_to_html, markdown_to_plain_text};
pub use docx::{markdown_to_docx, docx_to_markdown};
pub use pandoc::{markdown_to_odt, odt_to_markdown};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("pandoc not available: {0}")]
    PandocUnavailable(String),
    #[error("pandoc failed: {0}")]
    PandocFailed(String),
    #[error("docx error: {0}")]
    DocxError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
