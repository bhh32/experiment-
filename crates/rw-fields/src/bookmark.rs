//! Bookmark management.

use std::collections::HashMap;

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
}
