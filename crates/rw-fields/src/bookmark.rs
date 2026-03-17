//! Bookmark management.

use std::collections::HashMap;
use rw_document::Document;

/// Manages all bookmarks in a document.
#[derive(Debug, Clone, Default)]
pub struct BookmarkManager {
    bookmarks: HashMap<String, BookmarkInfo>,
}

#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    /// Page number where the bookmark is located (computed during layout)
    pub page_number: Option<usize>,
    /// The text content of the bookmark
    pub text: Option<String>,
    /// Whether this is a system bookmark (e.g., _Toc entries)
    pub system: bool,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace a bookmark.
    pub fn add_bookmark(&mut self, name: String, info: BookmarkInfo) {
        self.bookmarks.insert(name, info);
    }

    /// Remove a bookmark by name. Returns the removed info if present.
    pub fn remove_bookmark(&mut self, name: &str) -> Option<BookmarkInfo> {
        self.bookmarks.remove(name)
    }

    /// Retrieve a bookmark by name.
    pub fn get_bookmark(&self, name: &str) -> Option<&BookmarkInfo> {
        self.bookmarks.get(name)
    }

    /// List all bookmark names.
    pub fn list_bookmarks(&self) -> Vec<&str> {
        self.bookmarks.keys().map(|s| s.as_str()).collect()
    }

    // --- Legacy API kept for compatibility ---

    pub fn add(&mut self, name: String, info: BookmarkInfo) {
        self.bookmarks.insert(name, info);
    }

    pub fn get(&self, name: &str) -> Option<&BookmarkInfo> {
        self.bookmarks.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<BookmarkInfo> {
        self.bookmarks.remove(name)
    }

    pub fn all_names(&self) -> Vec<&str> {
        self.bookmarks.keys().map(|s| s.as_str()).collect()
    }

    /// Scan the document for bookmark markers and populate the manager.
    pub fn scan_document(&mut self, doc: &Document) {
        use rw_document::{Block, Inline};

        for section in &doc.sections {
            for block in &section.content {
                self.scan_block(block);
            }
        }
    }

    fn scan_block(&mut self, block: &rw_document::Block) {
        use rw_document::Block;
        use rw_document::Inline;

        match block {
            Block::Paragraph(para) => {
                let text = para.plain_text();
                for inline in &para.content {
                    if let Inline::BookmarkStart(mark) = inline {
                        self.bookmarks.entry(mark.name.clone()).or_insert(BookmarkInfo {
                            name: mark.name.clone(),
                            page_number: None,
                            text: Some(text.clone()),
                            system: mark.name.starts_with('_'),
                        });
                    }
                }
            }
            Block::Table(tbl) => {
                for cell in &tbl.cells {
                    for b in &cell.content {
                        self.scan_block(b);
                    }
                }
            }
            _ => {}
        }
    }
}
