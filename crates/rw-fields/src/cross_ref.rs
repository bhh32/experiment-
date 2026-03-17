//! Cross-reference fields.

use rw_document::inline::CrossRefType;
use rw_document::Document;
use crate::bookmark::BookmarkManager;

/// What to display for a cross-reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossRefDisplay {
    /// Show the referenced text content
    Text,
    /// Show the page number
    PageNumber,
    /// Show "above" or "below" relative to current position
    AboveBelow,
    /// Show the paragraph number (for numbered paragraphs)
    ParagraphNumber,
    /// Show the heading number
    HeadingNumber,
    /// Show the full context (e.g., "Figure 3")
    NumberedCaption,
    /// Show only the caption text
    CaptionText,
}

/// Manages cross-references within a document.
#[derive(Debug, Default)]
pub struct CrossRefManager {
    bookmarks: BookmarkManager,
}

impl CrossRefManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the reference table from the document.
    pub fn scan(&mut self, doc: &Document) {
        self.bookmarks.scan_document(doc);
    }

    /// Resolve a cross-reference by type and target bookmark ID.
    ///
    /// Returns the display string for the reference, or `None` if the
    /// target cannot be found.
    pub fn resolve_reference(
        &self,
        ref_type: CrossRefType,
        target_id: &str,
        _doc: &Document,
    ) -> Option<String> {
        match ref_type {
            CrossRefType::Text => {
                let info = self.bookmarks.get_bookmark(target_id)?;
                info.text.clone().or_else(|| Some(target_id.to_string()))
            }
            CrossRefType::PageNumber => {
                let info = self.bookmarks.get_bookmark(target_id)?;
                Some(
                    info.page_number
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                )
            }
            CrossRefType::Above => Some("above".to_string()),
            CrossRefType::Below => Some("below".to_string()),
            CrossRefType::ParagraphNumber => {
                // Without numbering state we return the target name
                Some(format!("[{}]", target_id))
            }
        }
    }
}
