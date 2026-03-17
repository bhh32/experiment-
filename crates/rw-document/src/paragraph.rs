use crate::inline::Inline;
use crate::properties::ParagraphProperties;
use crate::ElementId;
use serde::{Deserialize, Serialize};

/// A paragraph — a block-level element containing inline content.
///
/// Paragraphs are the primary container for text content.
/// Each paragraph has formatting properties and a list of inline
/// elements (text runs, images, fields, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// Unique identifier
    pub id: ElementId,
    /// Paragraph formatting properties
    pub properties: ParagraphProperties,
    /// Inline content elements
    pub content: Vec<Inline>,
}

impl Paragraph {
    /// Create an empty paragraph with default properties.
    pub fn new() -> Self {
        Self {
            id: ElementId::new(),
            properties: ParagraphProperties::default(),
            content: Vec::new(),
        }
    }

    /// Create a paragraph with a single text run.
    pub fn with_text(text: impl Into<String>) -> Self {
        let mut para = Self::new();
        para.content.push(Inline::Text(crate::TextRun::new(text)));
        para
    }

    /// Create a paragraph with a style applied.
    pub fn with_style(style_name: impl Into<String>) -> Self {
        let mut para = Self::new();
        para.properties.paragraph_style = Some(style_name.into());
        para
    }

    /// Append a text run to this paragraph.
    pub fn push_text(&mut self, text: impl Into<String>) {
        self.content.push(Inline::Text(crate::TextRun::new(text)));
    }

    /// Append any inline element to this paragraph.
    pub fn push_inline(&mut self, inline: Inline) {
        self.content.push(inline);
    }

    /// Returns true if the paragraph has no content.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Extract the plain text from all text runs in this paragraph.
    pub fn plain_text(&self) -> String {
        let mut result = String::new();
        for inline in &self.content {
            match inline {
                Inline::Text(run) => result.push_str(&run.text),
                Inline::Tab => result.push('\t'),
                Inline::NonBreakingSpace => result.push('\u{00A0}'),
                Inline::Break(crate::inline::BreakType::Line) => result.push('\n'),
                _ => {}
            }
        }
        result
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::new()
    }
}
