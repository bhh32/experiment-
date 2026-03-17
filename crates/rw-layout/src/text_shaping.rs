//! Text shaping and measurement using cosmic-text.
//!
//! This module wraps cosmic-text to provide text shaping, font
//! resolution, and measurement facilities for the layout engine.

/// Configuration for text shaping.
#[derive(Debug, Clone)]
pub struct ShapingConfig {
    /// Default font family
    pub default_font: String,
    /// Default font size in points
    pub default_size: f64,
    /// DPI for measurements
    pub dpi: f64,
}

impl Default for ShapingConfig {
    fn default() -> Self {
        Self {
            default_font: "Liberation Serif".to_string(),
            default_size: 12.0,
            dpi: 96.0,
        }
    }
}
