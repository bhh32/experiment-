//! Page navigator — thumbnail sidebar for page navigation.

/// State for the page navigator / thumbnail sidebar widget.
#[derive(Debug, Clone)]
pub struct PageNavigatorState {
    /// Total number of pages in the document
    pub total_pages: usize,
    /// Currently visible / active page (0-based)
    pub current_page: usize,
    /// Size of each thumbnail in pixels
    pub thumbnail_size: ThumbnailSize,
    /// Whether the navigator is visible
    pub visible: bool,
    /// Cached page thumbnails (page_index -> image data placeholder)
    pub thumbnails: Vec<PageThumbnail>,
}

impl Default for PageNavigatorState {
    fn default() -> Self {
        Self {
            total_pages: 0,
            current_page: 0,
            thumbnail_size: ThumbnailSize::Medium,
            visible: false,
            thumbnails: Vec::new(),
        }
    }
}

impl PageNavigatorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigate to a specific page.
    pub fn go_to_page(&mut self, page: usize) {
        if page < self.total_pages {
            self.current_page = page;
        }
    }

    /// Go to the next page (clamped to total_pages - 1).
    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages {
            self.current_page += 1;
        }
    }

    /// Go to the previous page (clamped to 0).
    pub fn prev_page(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
    }

    /// Update the page count, removing stale thumbnails.
    pub fn set_page_count(&mut self, count: usize) {
        self.total_pages = count;
        self.thumbnails.resize_with(count, PageThumbnail::placeholder);
        if self.current_page >= count && count > 0 {
            self.current_page = count - 1;
        }
    }

    /// Mark a thumbnail as dirty (needs re-rendering).
    pub fn invalidate_thumbnail(&mut self, page: usize) {
        if let Some(thumb) = self.thumbnails.get_mut(page) {
            thumb.dirty = true;
        }
    }

    /// Invalidate all thumbnails.
    pub fn invalidate_all(&mut self) {
        for thumb in &mut self.thumbnails {
            thumb.dirty = true;
        }
    }
}

/// Thumbnail size presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailSize {
    Small,
    Medium,
    Large,
}

impl ThumbnailSize {
    /// Return (width, height) in pixels for this preset.
    pub fn pixels(&self) -> (u32, u32) {
        match self {
            Self::Small => (80, 110),
            Self::Medium => (120, 165),
            Self::Large => (180, 248),
        }
    }
}

/// Metadata for a single page thumbnail.
#[derive(Debug, Clone)]
pub struct PageThumbnail {
    /// Page index (0-based)
    pub page_index: usize,
    /// Whether this thumbnail needs to be re-rendered
    pub dirty: bool,
    /// Raw pixel data (RGBA, row-major) — None if not yet rendered
    pub data: Option<Vec<u8>>,
}

impl PageThumbnail {
    pub fn placeholder() -> Self {
        Self {
            page_index: 0,
            dirty: true,
            data: None,
        }
    }
}
