//! Render theme — colors and styles for the document canvas.

/// Visual theme for the document rendering area.
#[derive(Debug, Clone)]
pub struct RenderTheme {
    /// Background color of the area behind pages ("desk")
    pub desk_color: String,
    /// Page background color
    pub page_color: String,
    /// Selection highlight color
    pub selection_color: String,
    /// Cursor color
    pub cursor_color: String,
    /// Find/replace highlight color
    pub find_highlight_color: String,
    /// Spelling error underline color
    pub spell_error_color: String,
    /// Grammar error underline color
    pub grammar_error_color: String,
    /// Comment highlight color
    pub comment_highlight_color: String,
    /// Track changes insertion color
    pub track_insert_color: String,
    /// Track changes deletion color
    pub track_delete_color: String,
    /// Page shadow color
    pub page_shadow_color: String,
    /// Ruler background color
    pub ruler_background: String,
    /// Ruler text color
    pub ruler_text_color: String,
    /// Margin guide color
    pub margin_guide_color: String,
}

impl Default for RenderTheme {
    fn default() -> Self {
        Self {
            desk_color: "#E0E0E0".to_string(),
            page_color: "#FFFFFF".to_string(),
            selection_color: "#3399FF40".to_string(),
            cursor_color: "#000000".to_string(),
            find_highlight_color: "#FFFF0080".to_string(),
            spell_error_color: "#FF0000".to_string(),
            grammar_error_color: "#0000FF".to_string(),
            comment_highlight_color: "#FFFF9960".to_string(),
            track_insert_color: "#00880080".to_string(),
            track_delete_color: "#FF000080".to_string(),
            page_shadow_color: "#00000040".to_string(),
            ruler_background: "#F0F0F0".to_string(),
            ruler_text_color: "#666666".to_string(),
            margin_guide_color: "#CCCCCC".to_string(),
        }
    }
}
