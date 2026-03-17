//! Text shaping and measurement.
//!
//! This module provides text shaping, font resolution, and measurement
//! facilities for the layout engine. Uses a simple character-based
//! approximation for width estimation.

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

/// Measured text dimensions.
#[derive(Debug, Clone, Copy)]
pub struct TextMetrics {
    /// Total advance width of the text
    pub width: f64,
    /// Total line height (ascent + descent)
    pub height: f64,
    /// Distance from baseline to top of glyphs
    pub ascent: f64,
    /// Distance from baseline to bottom of glyphs
    pub descent: f64,
}

impl TextMetrics {
    /// Create zeroed metrics.
    pub fn zero() -> Self {
        Self { width: 0.0, height: 0.0, ascent: 0.0, descent: 0.0 }
    }
}

/// Measure the dimensions of a text string with the given shaping config.
///
/// Uses a simple monospace approximation:
/// - width  = char_count × font_size × 0.6
/// - height = font_size × 1.2
/// - ascent = font_size × 0.8 (roughly 80 % of em above baseline)
/// - descent = font_size × 0.2
pub fn measure_text(text: &str, config: &ShapingConfig) -> TextMetrics {
    let char_count = text.chars().count() as f64;
    let font_size = config.default_size;

    TextMetrics {
        width: char_count * font_size * 0.6,
        height: font_size * 1.2,
        ascent: font_size * 0.8,
        descent: font_size * 0.2,
    }
}

/// Measure a text string at an explicit font size (ignores config size).
pub fn measure_text_at_size(text: &str, font_size: f64) -> TextMetrics {
    let char_count = text.chars().count() as f64;
    TextMetrics {
        width: char_count * font_size * 0.6,
        height: font_size * 1.2,
        ascent: font_size * 0.8,
        descent: font_size * 0.2,
    }
}
