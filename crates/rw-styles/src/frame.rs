use crate::{StyleBase, StyleCategory};
use serde::{Deserialize, Serialize};

/// A frame style definition.
///
/// Frame styles control the appearance of text frames, image frames,
/// and OLE object frames. They define borders, background, wrap mode,
/// and positioning defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStyle {
    pub base: StyleBase,
    pub properties: FrameStyleProperties,
}

impl FrameStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::Frame),
            properties: FrameStyleProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameStyleProperties {
    /// Border style
    pub border_style: Option<String>,
    /// Border width in twips
    pub border_width_twips: Option<i32>,
    /// Border color
    pub border_color: Option<String>,
    /// Background color
    pub background: Option<String>,
    /// Padding inside the frame in twips
    pub padding_twips: Option<i32>,
    /// Shadow
    pub shadow: Option<bool>,
    /// Shadow color
    pub shadow_color: Option<String>,
    /// Wrap mode: "none", "square", "tight", "through", "top-and-bottom"
    pub wrap_mode: Option<String>,
    /// Horizontal alignment: "left", "center", "right"
    pub horizontal_alignment: Option<String>,
    /// Vertical alignment: "top", "center", "bottom"
    pub vertical_alignment: Option<String>,
}
