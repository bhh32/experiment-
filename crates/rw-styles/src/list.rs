use crate::{StyleBase, StyleCategory};
use serde::{Deserialize, Serialize};

/// A list/numbering style definition.
///
/// List styles define the appearance of numbered and bulleted lists
/// across up to 10 outline levels. Each level specifies its number
/// format, bullet character, indentation, and text properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStyle {
    pub base: StyleBase,
    pub levels: Vec<ListLevelStyle>,
}

impl ListStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::List),
            levels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLevelStyle {
    /// Level index (0-9)
    pub level: u8,
    /// Number format: "decimal", "lower-alpha", "upper-alpha",
    /// "lower-roman", "upper-roman", "bullet", "none"
    pub format: String,
    /// Text template (e.g., "%1.", "(%1)", "Section %1.%2")
    pub level_text: String,
    /// Starting number
    pub start: u32,
    /// Bullet character (for bullet format)
    pub bullet_char: Option<String>,
    /// Bullet font
    pub bullet_font: Option<String>,
    /// Left indent in twips
    pub indent_twips: i32,
    /// Text indent (hanging indent) in twips
    pub text_indent_twips: i32,
    /// Suffix after the number: "tab", "space", "nothing"
    pub suffix: String,
    /// Character style for the number/bullet
    pub char_style: Option<String>,
}
