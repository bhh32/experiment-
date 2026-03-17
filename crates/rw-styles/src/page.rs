use crate::{StyleBase, StyleCategory};
use serde::{Deserialize, Serialize};

/// A page style definition.
///
/// Page styles control the physical layout of pages: size, margins,
/// orientation, columns, headers/footers, background, and borders.
/// Common page styles: "Default", "First Page", "Left Page",
/// "Right Page", "Landscape", "Envelope".
///
/// Page styles support the "Next Style" property for automatic
/// sequencing (e.g., "First Page" -> "Default").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStyle {
    pub base: StyleBase,
    pub properties: PageStyleProperties,
}

impl PageStyle {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: StyleBase::new(name, StyleCategory::Page),
            properties: PageStyleProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageStyleProperties {
    /// Page width in twips
    pub width_twips: Option<i32>,
    /// Page height in twips
    pub height_twips: Option<i32>,
    /// Orientation: "portrait" or "landscape"
    pub orientation: Option<String>,
    /// Margins
    pub margin_top_twips: Option<i32>,
    pub margin_bottom_twips: Option<i32>,
    pub margin_left_twips: Option<i32>,
    pub margin_right_twips: Option<i32>,
    pub margin_header_twips: Option<i32>,
    pub margin_footer_twips: Option<i32>,
    /// Gutter width
    pub gutter_twips: Option<i32>,
    /// Mirror margins for facing pages
    pub mirror_margins: Option<bool>,
    /// Number of columns
    pub column_count: Option<u32>,
    /// Column spacing
    pub column_spacing_twips: Option<i32>,
    /// Background color
    pub background_color: Option<String>,
    /// Page numbering format
    pub page_number_format: Option<String>,
    /// Enable header
    pub header_enabled: Option<bool>,
    /// Enable footer
    pub footer_enabled: Option<bool>,
    /// Different first page header/footer
    pub different_first_page: Option<bool>,
    /// Different even/odd page headers/footers
    pub different_even_odd: Option<bool>,
}
