//! Clipboard operations — cut, copy, paste with rich content support.

use rw_document::block::Block;

/// Data that can be placed on or read from the clipboard.
#[derive(Debug, Clone)]
pub enum ClipboardContent {
    /// Plain text
    PlainText(String),
    /// Rich content (document blocks)
    Rich(Vec<Block>),
    /// HTML text
    Html(String),
    /// RTF text
    Rtf(String),
}

/// Clipboard operation type.
#[derive(Debug, Clone, Copy)]
pub enum ClipboardOp {
    Cut,
    Copy,
    Paste,
    PasteSpecial,
}
