use crate::paragraph::Paragraph;
use crate::ElementId;
use serde::{Deserialize, Serialize};

/// Block-level elements that make up the body of a document section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    /// A paragraph of text content
    Paragraph(Paragraph),

    /// A table (structure defined in rw-tables, referenced here as opaque data)
    Table(TableBlock),

    /// A floating/anchored frame containing block content
    Frame(FrameBlock),

    /// A horizontal rule / separator
    HorizontalRule(HorizontalRule),

    /// A section — a subdivison of the document that can have its own
    /// column layout, write protection, or linked content
    SubSection(SubSection),

    /// A table of contents, index, or bibliography block
    GeneratedContent(GeneratedContent),
}

/// A table block — the actual table data structure is defined in rw-tables.
/// This is a thin wrapper that carries the table data as a serializable structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBlock {
    pub id: ElementId,
    /// Number of rows
    pub rows: u32,
    /// Number of columns
    pub cols: u32,
    /// The table cells, stored as a flat Vec in row-major order.
    /// Each cell contains a list of blocks (paragraphs, nested tables, etc.)
    pub cells: Vec<TableCell>,
    /// Table-wide properties
    pub properties: TableProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub id: ElementId,
    /// The content of this cell
    pub content: Vec<Block>,
    /// Cell-specific properties
    pub properties: TableCellProperties,
    /// How many columns this cell spans
    pub col_span: u32,
    /// How many rows this cell spans
    pub row_span: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableProperties {
    /// Table width (None = auto)
    pub width: Option<TableWidth>,
    /// Table alignment
    pub alignment: Option<crate::properties::Alignment>,
    /// Default cell margins
    pub default_cell_margins: Option<CellMargins>,
    /// Table borders
    pub borders: Option<TableBorders>,
    /// Table style reference
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableWidth {
    Auto,
    Fixed(crate::Twips),
    Percent(f64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableCellProperties {
    /// Cell width
    pub width: Option<TableWidth>,
    /// Cell margins/padding
    pub margins: Option<CellMargins>,
    /// Cell borders
    pub borders: Option<TableBorders>,
    /// Cell background/shading
    pub background: Option<crate::Color>,
    /// Vertical alignment of content
    pub vertical_alignment: Option<VerticalAlignment>,
    /// Text direction
    pub text_direction: Option<CellTextDirection>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CellMargins {
    pub top: crate::Twips,
    pub bottom: crate::Twips,
    pub left: crate::Twips,
    pub right: crate::Twips,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBorders {
    pub top: Option<crate::properties::BorderLine>,
    pub bottom: Option<crate::properties::BorderLine>,
    pub left: Option<crate::properties::BorderLine>,
    pub right: Option<crate::properties::BorderLine>,
    pub inside_horizontal: Option<crate::properties::BorderLine>,
    pub inside_vertical: Option<crate::properties::BorderLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellTextDirection {
    Horizontal,
    VerticalTopToBottom,
    VerticalBottomToTop,
}

/// A floating frame containing block content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameBlock {
    pub id: ElementId,
    pub content: Vec<Block>,
    pub width: Option<crate::Twips>,
    pub height: Option<crate::Twips>,
    pub anchor: AnchorType,
    pub wrap: WrapMode,
    pub position: FramePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorType {
    /// Anchored to the page
    Page,
    /// Anchored to a paragraph
    Paragraph,
    /// Anchored to a character position
    Character,
    /// Inline (as character)
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapMode {
    None,
    Square,
    Tight,
    Through,
    TopAndBottom,
    BehindText,
    InFrontOfText,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FramePosition {
    pub horizontal: crate::Twips,
    pub vertical: crate::Twips,
    pub relative_horizontal: RelativePosition,
    pub relative_vertical: RelativePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelativePosition {
    Page,
    Margin,
    Column,
    Paragraph,
    Character,
    Line,
}

/// A horizontal rule / separator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalRule {
    pub id: ElementId,
    pub width_percent: f64,
    pub height: crate::Twips,
    pub color: crate::Color,
    pub alignment: crate::properties::Alignment,
}

/// A document subsection (for column changes, protected sections, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubSection {
    pub id: ElementId,
    pub name: Option<String>,
    pub content: Vec<Block>,
    pub properties: SubSectionProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubSectionProperties {
    pub columns: Option<crate::properties::ColumnLayout>,
    pub write_protected: bool,
    pub hidden: bool,
}

/// Auto-generated content blocks (TOC, index, bibliography).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedContent {
    pub id: ElementId,
    pub content_type: GeneratedContentType,
    /// The rendered content (generated on update)
    pub rendered_blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratedContentType {
    TableOfContents {
        /// Heading levels to include (e.g., 1..=3)
        from_level: u8,
        to_level: u8,
        /// Include page numbers
        show_page_numbers: bool,
        /// Right-aligned page numbers with leader
        right_align_page_numbers: bool,
        /// Tab leader style
        leader: crate::properties::TabLeader,
    },
    TableOfFigures {
        caption_label: String,
    },
    AlphabeticalIndex,
    Bibliography,
}
