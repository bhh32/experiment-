//! Color picker widget.
//!
//! A full-featured color picker with:
//! - Theme colors (from document theme)
//! - Standard colors
//! - Recent colors
//! - Custom color dialog (HSL/RGB/Hex input)

/// Standard colors available in the picker.
pub const STANDARD_COLORS: &[(&str, &str)] = &[
    ("#FF0000", "Red"),
    ("#FF8000", "Orange"),
    ("#FFFF00", "Yellow"),
    ("#00FF00", "Green"),
    ("#00FFFF", "Cyan"),
    ("#0000FF", "Blue"),
    ("#8000FF", "Purple"),
    ("#FF00FF", "Magenta"),
    ("#000000", "Black"),
    ("#404040", "Dark Gray"),
    ("#808080", "Gray"),
    ("#C0C0C0", "Silver"),
    ("#FFFFFF", "White"),
];

/// Color picker state.
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// Currently selected color (hex)
    pub selected: Option<String>,
    /// Recent colors used
    pub recent: Vec<String>,
    /// Maximum number of recent colors to remember
    pub max_recent: usize,
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            selected: None,
            recent: Vec::new(),
            max_recent: 10,
        }
    }
}
