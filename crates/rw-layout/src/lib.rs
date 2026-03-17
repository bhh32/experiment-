//! # rw-layout
//!
//! The document layout engine for Rust Writer.
//!
//! This crate takes the logical document model (from rw-document) and
//! computes the physical layout: where each glyph, line, paragraph,
//! image, table, and page boundary falls on the rendered pages.
//!
//! ## Pipeline
//!
//! 1. **Style Resolution**: Resolve all inherited styles to compute
//!    effective properties for each element.
//! 2. **Inline Layout**: Shape text runs into glyph runs, measure widths,
//!    compute line breaks using Unicode line breaking algorithm.
//! 3. **Block Layout**: Stack paragraphs and other block elements vertically,
//!    applying spacing, indentation, and borders.
//! 4. **Page Layout**: Fit blocks into page columns, computing page breaks,
//!    handling headers/footers, floating objects, and footnotes.
//! 5. **Output**: Produce a `LayoutResult` containing positioned pages
//!    with their content ready for rendering.
//!
//! ## Key Types
//!
//! - `LayoutEngine`: The main entry point that performs layout
//! - `LayoutResult`: The complete layout output
//! - `LayoutPage`: A single laid-out page
//! - `LayoutLine`: A single line of text with positioned glyph runs
//! - `LayoutBox`: A positioned rectangular region

pub mod engine;
pub mod line;
pub mod page;
pub mod result;
pub mod text_shaping;

pub use engine::LayoutEngine;
pub use result::LayoutResult;

/// A rectangle in layout coordinates (origin at top-left of page).
/// All values in twips.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }
}

/// A point in layout coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPoint {
    pub x: f64,
    pub y: f64,
}
