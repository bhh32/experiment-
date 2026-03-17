//! Style gallery widget — visual preview of available styles.

/// A single entry in the style gallery.
#[derive(Debug, Clone)]
pub struct StyleGalleryEntry {
    /// Style name (key in the catalog)
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Short sample text to render in the preview
    pub preview_text: String,
    /// CSS-like font size hint for the preview (in points)
    pub preview_font_size: f64,
    /// Whether this style is bold in the preview
    pub preview_bold: bool,
    /// Whether this style is italic in the preview
    pub preview_italic: bool,
    /// Foreground color hint ("#RRGGBB") for the preview
    pub preview_color: String,
}

impl StyleGalleryEntry {
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            preview_text: "AaBbCcDd".to_string(),
            preview_font_size: 12.0,
            preview_bold: false,
            preview_italic: false,
            preview_color: "#000000".to_string(),
        }
    }
}

/// State of the Quick Styles gallery widget.
#[derive(Debug, Clone)]
pub struct StyleGalleryState {
    /// All styles shown in the gallery (in display order)
    pub styles: Vec<StyleGalleryEntry>,
    /// Name of the currently applied style
    pub selected: Option<String>,
    /// How many styles are visible without scrolling
    pub visible_count: usize,
    /// Scroll offset (first visible style index)
    pub scroll_offset: usize,
}

impl Default for StyleGalleryState {
    fn default() -> Self {
        Self {
            styles: default_gallery_entries(),
            selected: Some("Normal".to_string()),
            visible_count: 8,
            scroll_offset: 0,
        }
    }
}

impl StyleGalleryState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a style by name.
    pub fn select(&mut self, name: impl Into<String>) {
        self.selected = Some(name.into());
    }

    /// Scroll forward by `n` entries.
    pub fn scroll_forward(&mut self, n: usize) {
        let max_offset = self.styles.len().saturating_sub(self.visible_count);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
    }

    /// Scroll back by `n` entries.
    pub fn scroll_back(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Return the slice of entries that are currently visible.
    pub fn visible_entries(&self) -> &[StyleGalleryEntry] {
        let end = (self.scroll_offset + self.visible_count).min(self.styles.len());
        &self.styles[self.scroll_offset..end]
    }
}

/// Build the default gallery entries for the built-in styles.
pub fn default_gallery_entries() -> Vec<StyleGalleryEntry> {
    vec![
        {
            let mut e = StyleGalleryEntry::new("Normal", "Normal");
            e.preview_text = "Normal".to_string();
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Heading 1", "Heading 1");
            e.preview_text = "Heading 1".to_string();
            e.preview_font_size = 16.0;
            e.preview_bold = true;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Heading 2", "Heading 2");
            e.preview_text = "Heading 2".to_string();
            e.preview_font_size = 13.0;
            e.preview_bold = true;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Heading 3", "Heading 3");
            e.preview_text = "Heading 3".to_string();
            e.preview_font_size = 12.0;
            e.preview_bold = true;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Title", "Title");
            e.preview_text = "Title".to_string();
            e.preview_font_size = 28.0;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Subtitle", "Subtitle");
            e.preview_text = "Subtitle".to_string();
            e.preview_font_size = 14.0;
            e.preview_italic = true;
            e.preview_color = "#5A5A5A".to_string();
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Body Text", "Body Text");
            e.preview_text = "Body Text".to_string();
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Quote", "Quote");
            e.preview_text = "\"Quote\"".to_string();
            e.preview_italic = true;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("Caption", "Caption");
            e.preview_text = "Caption".to_string();
            e.preview_font_size = 10.0;
            e.preview_italic = true;
            e
        },
        {
            let mut e = StyleGalleryEntry::new("List Paragraph", "List Paragraph");
            e.preview_text = "• List Item".to_string();
            e
        },
    ]
}
