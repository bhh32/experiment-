//! # rw-render
//!
//! GPU-accelerated document renderer for Rust Writer.
//!
//! Takes the layout result from rw-layout and renders it to the screen
//! using iced's Canvas widget with wgpu backend. This provides:
//!
//! - Smooth scrolling and zooming
//! - Crisp text rendering at any zoom level
//! - Hardware-accelerated drawing of shapes, images, and effects
//! - Page shadow and gap rendering
//! - Selection highlighting and cursor drawing
//! - Margin guides and ruler marks
//!
//! ## Architecture
//!
//! The renderer operates in screen coordinates (pixels). It receives
//! the layout (in twips/points) and a viewport/zoom level, and draws
//! only the visible portion of the document.

pub mod canvas;
pub mod viewport;
pub mod theme;

pub use viewport::Viewport;

/// The render configuration.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Zoom level (1.0 = 100%)
    pub zoom: f64,
    /// DPI of the display
    pub dpi: f64,
    /// Whether to show page boundaries
    pub show_page_boundaries: bool,
    /// Whether to show text boundaries (for debugging)
    pub show_text_boundaries: bool,
    /// Whether to show non-printing characters (paragraph marks, spaces, etc.)
    pub show_formatting_marks: bool,
    /// Whether to show the ruler
    pub show_rulers: bool,
    /// Background color behind pages (the "desk" color)
    pub desk_color: String,
    /// Page shadow enabled
    pub page_shadow: bool,
    /// Gap between pages in points
    pub page_gap: f64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            dpi: 96.0,
            show_page_boundaries: true,
            show_text_boundaries: false,
            show_formatting_marks: false,
            show_rulers: true,
            desk_color: "#808080".to_string(),
            page_shadow: true,
            page_gap: 20.0,
        }
    }
}
