use crate::LayoutRect;
use rw_document::ElementId;

/// A laid-out line of text on a page.
#[derive(Debug, Clone)]
pub struct LayoutLine {
    /// Bounding box of this line (position relative to page origin)
    pub bounds: LayoutRect,
    /// The glyph runs that make up this line
    pub runs: Vec<GlyphRun>,
    /// Baseline position (Y offset from top of line)
    pub baseline: f64,
    /// ID of the paragraph this line belongs to
    pub paragraph_id: ElementId,
    /// Line index within the paragraph
    pub line_in_paragraph: usize,
}

/// A run of shaped glyphs with uniform formatting.
#[derive(Debug, Clone)]
pub struct GlyphRun {
    /// Bounding box of this run (position relative to page origin)
    pub bounds: LayoutRect,
    /// The shaped glyphs
    pub glyphs: Vec<PositionedGlyph>,
    /// Font family for this run
    pub font_family: String,
    /// Font size in points
    pub font_size: f64,
    /// Whether the run is bold
    pub bold: bool,
    /// Whether the run is italic
    pub italic: bool,
    /// Text color (RGBA hex)
    pub color: String,
    /// The source text run ID
    pub text_run_id: ElementId,
    /// The source text for hit testing
    pub text: String,
}

/// A single positioned glyph.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// Glyph ID (font-specific)
    pub glyph_id: u32,
    /// X position relative to run origin
    pub x: f64,
    /// Y position relative to run origin (baseline-relative)
    pub y: f64,
    /// Advance width
    pub advance: f64,
    /// Byte offset in the source text
    pub byte_offset: usize,
}
