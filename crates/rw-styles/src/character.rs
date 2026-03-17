use crate::paragraph::CharacterStyleProperties;
use crate::{StyleBase, StyleCategory};
use serde::{Deserialize, Serialize};

/// A character style definition.
///
/// Character styles are applied to text spans within paragraphs to
/// override the paragraph style's character properties.
/// Common character styles: "Default Paragraph Font", "Strong",
/// "Emphasis", "Hyperlink", "Footnote Reference", etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub base: StyleBase,
    pub properties: CharacterStyleProperties,
}

impl CharacterStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::Character),
            properties: CharacterStyleProperties::default(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.base.parent = Some(parent.into());
        self
    }
}
