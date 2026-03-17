use crate::block::Block;
use crate::properties::PageProperties;
use crate::ElementId;
use serde::{Deserialize, Serialize};

/// A document section — a region of the document with uniform page layout.
///
/// Sections allow different parts of the document to have different
/// page sizes, orientations, margins, headers/footers, and column layouts.
/// A typical document has one section, but complex documents may have many
/// (e.g., a title page in portrait, followed by content in landscape with
/// two columns, followed by an appendix in portrait).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Unique identifier
    pub id: ElementId,
    /// Page layout properties for this section
    pub page_properties: PageProperties,
    /// How this section starts relative to the previous one
    pub start_type: SectionStartType,
    /// Block-level content in this section
    pub content: Vec<Block>,
    /// Header definitions for this section
    pub headers: HeaderFooterSet,
    /// Footer definitions for this section
    pub footers: HeaderFooterSet,
    /// Whether to restart page numbering in this section
    pub restart_page_numbering: Option<u32>,
    /// Line numbering settings
    pub line_numbering: Option<LineNumbering>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            page_properties: PageProperties::default(),
            start_type: SectionStartType::NextPage,
            content: Vec::new(),
            headers: HeaderFooterSet::default(),
            footers: HeaderFooterSet::default(),
            restart_page_numbering: None,
            line_numbering: None,
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

/// How a new section starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionStartType {
    /// Continue on the same page
    Continuous,
    /// Start on the next page
    NextPage,
    /// Start on the next even page
    EvenPage,
    /// Start on the next odd page
    OddPage,
}

/// A set of headers or footers for a section.
/// Sections can have different headers for the first page,
/// even pages, and odd pages (the default).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeaderFooterSet {
    /// Default header (used for all pages unless overridden)
    pub default: Option<HeaderFooter>,
    /// First page header (if different from default)
    pub first_page: Option<HeaderFooter>,
    /// Even page header (if different from default, for facing pages)
    pub even_page: Option<HeaderFooter>,
}

/// A header or footer region containing block content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooter {
    pub id: ElementId,
    /// The content of the header/footer
    pub content: Vec<Block>,
}

impl HeaderFooter {
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            content: Vec::new(),
        }
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        let mut hf = Self::new();
        hf.content.push(Block::Paragraph(crate::Paragraph::with_text(text)));
        hf
    }
}

impl Default for HeaderFooter {
    fn default() -> Self {
        Self::new()
    }
}

/// Line numbering configuration for a section.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LineNumbering {
    /// Starting number
    pub start: u32,
    /// Increment (show every Nth line)
    pub increment: u32,
    /// Distance from text
    pub distance: crate::Twips,
    /// Whether to restart numbering on each page
    pub restart_on_page: bool,
}
