use crate::properties::CharacterProperties;
use crate::ElementId;
use serde::{Deserialize, Serialize};

/// A contiguous run of text with uniform character formatting.
///
/// Text runs are the leaf nodes of the inline content tree.
/// Each run contains a string of text and the formatting properties
/// that apply to that text. When formatting changes mid-paragraph,
/// a new text run is created at the boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    /// Unique identifier
    pub id: ElementId,
    /// The text content
    pub text: String,
    /// Character formatting applied to this run
    pub properties: CharacterProperties,
}

impl TextRun {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: ElementId::new(),
            text: text.into(),
            properties: CharacterProperties::default(),
        }
    }

    pub fn with_properties(text: impl Into<String>, properties: CharacterProperties) -> Self {
        Self {
            id: ElementId::new(),
            text: text.into(),
            properties,
        }
    }

    /// Returns true if this run contains no text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the number of grapheme clusters in this run.
    pub fn grapheme_count(&self) -> usize {
        use unicode_segmentation::UnicodeSegmentation;
        self.text.graphemes(true).count()
    }

    /// Split this run at the given byte offset, returning the second half.
    pub fn split_at(&mut self, byte_offset: usize) -> TextRun {
        let second_text = self.text.split_off(byte_offset);
        TextRun {
            id: ElementId::new(),
            text: second_text,
            properties: self.properties.clone(),
        }
    }
}
