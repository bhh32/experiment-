/// The viewport — the visible portion of the document.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Horizontal scroll offset in screen pixels
    pub scroll_x: f64,
    /// Vertical scroll offset in screen pixels
    pub scroll_y: f64,
    /// Width of the viewport in screen pixels
    pub width: f64,
    /// Height of the viewport in screen pixels
    pub height: f64,
    /// Current zoom level (1.0 = 100%)
    pub zoom: f64,
}

impl Viewport {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// Convert a screen coordinate to a document coordinate.
    pub fn screen_to_doc(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        (
            (screen_x + self.scroll_x) / self.zoom,
            (screen_y + self.scroll_y) / self.zoom,
        )
    }

    /// Convert a document coordinate to a screen coordinate.
    pub fn doc_to_screen(&self, doc_x: f64, doc_y: f64) -> (f64, f64) {
        (
            doc_x * self.zoom - self.scroll_x,
            doc_y * self.zoom - self.scroll_y,
        )
    }

    /// Whether a document rectangle is visible in this viewport.
    pub fn is_visible(&self, doc_x: f64, doc_y: f64, doc_w: f64, doc_h: f64) -> bool {
        let (sx, sy) = self.doc_to_screen(doc_x, doc_y);
        let (sw, sh) = (doc_w * self.zoom, doc_h * self.zoom);
        sx + sw >= 0.0 && sx <= self.width && sy + sh >= 0.0 && sy <= self.height
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}
