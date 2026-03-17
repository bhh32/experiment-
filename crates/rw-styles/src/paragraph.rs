use crate::{StyleBase, StyleCategory, StyleName};
use serde::{Deserialize, Serialize};

/// A paragraph style definition.
///
/// Paragraph styles control the appearance of entire paragraphs.
/// They can specify both paragraph-level and character-level defaults.
/// Common paragraph styles include "Normal", "Heading 1-9", "Title",
/// "Subtitle", "Body Text", "Quote", etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub base: StyleBase,
    /// Paragraph formatting properties (alignment, spacing, indent, etc.)
    /// These are stored as raw key-value pairs to avoid circular dependency
    /// with rw-document. The actual property types are resolved at use time.
    pub properties: ParagraphStyleProperties,
    /// Default character properties for text in paragraphs with this style
    pub char_properties: CharacterStyleProperties,
}

impl ParagraphStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::Paragraph),
            properties: ParagraphStyleProperties::default(),
            char_properties: CharacterStyleProperties::default(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.base.parent = Some(parent.into());
        self
    }

    pub fn with_next(mut self, next: impl Into<String>) -> Self {
        self.base.next_style = Some(next.into());
        self
    }
}

/// Serializable paragraph formatting properties for styles.
/// Uses Option<T> for all fields to support inheritance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParagraphStyleProperties {
    pub alignment: Option<String>,
    pub indent_left_twips: Option<i32>,
    pub indent_right_twips: Option<i32>,
    pub indent_first_line_twips: Option<i32>,
    pub space_before_twips: Option<i32>,
    pub space_after_twips: Option<i32>,
    pub line_spacing_type: Option<String>,
    pub line_spacing_value: Option<f64>,
    pub keep_with_next: Option<bool>,
    pub keep_together: Option<bool>,
    pub page_break_before: Option<bool>,
    pub widow_control: Option<u32>,
    pub orphan_control: Option<u32>,
    pub outline_level: Option<u8>,
    pub suppress_line_numbers: Option<bool>,
    pub suppress_hyphenation: Option<bool>,
    pub text_direction: Option<String>,
    pub background_color: Option<String>,
    pub border_top: Option<String>,
    pub border_bottom: Option<String>,
    pub border_left: Option<String>,
    pub border_right: Option<String>,
}

/// Serializable character formatting properties for styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterStyleProperties {
    pub font_family: Option<String>,
    pub font_size_half_points: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<String>,
    pub strikethrough: Option<String>,
    pub color: Option<String>,
    pub highlight: Option<String>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub small_caps: Option<bool>,
    pub all_caps: Option<bool>,
    pub hidden: Option<bool>,
    pub spacing_twips: Option<i32>,
    pub kerning: Option<u32>,
    pub position_half_points: Option<i32>,
    pub scale_percent: Option<u32>,
    pub shadow: Option<bool>,
    pub outline: Option<bool>,
    pub language: Option<String>,
}
