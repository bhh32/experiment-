//! Table of contents generation.

use rw_document::Document;

/// Configuration for generating a table of contents.
#[derive(Debug, Clone)]
pub struct TocConfig {
    /// Minimum heading level to include (1 = Heading 1)
    pub from_level: u8,
    /// Maximum heading level to include
    pub to_level: u8,
    /// Show page numbers
    pub show_page_numbers: bool,
    /// Right-align page numbers
    pub right_align_numbers: bool,
    /// Leader character between text and page number
    pub leader: TocLeader,
    /// Use hyperlinks
    pub use_hyperlinks: bool,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            from_level: 1,
            to_level: 3,
            show_page_numbers: true,
            right_align_numbers: true,
            leader: TocLeader::Dot,
            use_hyperlinks: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TocLeader {
    None,
    Dot,
    Dash,
    Underscore,
}

/// A single entry in a table of contents.
#[derive(Debug, Clone)]
pub struct TocEntry {
    /// Heading level (1 = H1, 2 = H2, etc.)
    pub level: u8,
    /// The heading text
    pub text: String,
    /// Page number (0 if not yet laid out)
    pub page: usize,
}

/// Generates TOC entries by scanning headings in the document.
#[derive(Debug, Default)]
pub struct TocGenerator {
    config: TocConfig,
}

impl TocGenerator {
    pub fn new(config: TocConfig) -> Self {
        Self { config }
    }

    /// Scan the document for heading paragraphs and produce TOC entries.
    ///
    /// Headings are identified by their paragraph style name matching
    /// "Heading 1", "Heading 2", etc. (case-insensitive).
    pub fn generate_toc(&self, doc: &Document) -> Vec<TocEntry> {
        let mut entries = Vec::new();

        for section in &doc.sections {
            for block in &section.content {
                self.visit_block(block, &mut entries);
            }
        }

        entries
    }

    fn visit_block(&self, block: &rw_document::Block, entries: &mut Vec<TocEntry>) {
        use rw_document::Block;

        match block {
            Block::Paragraph(para) => {
                if let Some(style) = &para.properties.paragraph_style {
                    if let Some(level) = heading_level(style) {
                        if level >= self.config.from_level && level <= self.config.to_level {
                            entries.push(TocEntry {
                                level,
                                text: para.plain_text(),
                                page: 0, // unknown without layout
                            });
                        }
                    }
                }
            }
            Block::Table(tbl) => {
                for cell in &tbl.cells {
                    for b in &cell.content {
                        self.visit_block(b, entries);
                    }
                }
            }
            Block::SubSection(sub) => {
                for b in &sub.content {
                    self.visit_block(b, entries);
                }
            }
            _ => {}
        }
    }
}

/// Parse heading level from style name ("Heading 1" -> 1, "heading2" -> 2, etc.)
fn heading_level(style: &str) -> Option<u8> {
    let lower = style.to_lowercase();
    let _trimmed = lower.trim_start_matches("heading").trim_start_matches(' ').trim();
    // "Heading 1", "Heading1", "h1"
    if lower.starts_with("heading") {
        let num_part = lower
            .trim_start_matches("heading")
            .trim()
            .trim_start_matches(' ');
        num_part.parse::<u8>().ok()
    } else if lower.starts_with('h') && lower.len() == 2 {
        lower[1..].parse::<u8>().ok()
    } else {
        None
    }
}
