use crate::page::LayoutPage;

/// The complete result of laying out a document.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// The laid-out pages
    pub pages: Vec<LayoutPage>,
    /// Total number of pages
    pub page_count: usize,
}

impl LayoutResult {
    pub fn empty() -> Self {
        Self {
            pages: Vec::new(),
            page_count: 0,
        }
    }

    /// Get a specific page by index.
    pub fn get_page(&self, index: usize) -> Option<&LayoutPage> {
        self.pages.get(index)
    }
}
