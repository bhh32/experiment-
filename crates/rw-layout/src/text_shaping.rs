//! Text shaping and measurement using cosmic-text.
//!
//! This module wraps `cosmic_text::FontSystem` and `cosmic_text::Buffer`
//! to provide accurate, HarfBuzz-backed text shaping, font discovery,
//! and glyph measurement for the layout engine.

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, Weight,
};

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
    /// Total line height (ascent + descent + leading)
    pub height: f64,
    /// Distance from baseline to top of glyphs
    pub ascent: f64,
    /// Distance from baseline to bottom of glyphs
    pub descent: f64,
}

impl TextMetrics {
    /// Create zeroed metrics.
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            ascent: 0.0,
            descent: 0.0,
        }
    }
}

/// Shaped glyph information returned by the shaper.
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID (font-specific)
    pub glyph_id: u32,
    /// X position relative to the start of the run
    pub x: f64,
    /// Advance width of this glyph
    pub advance: f64,
    /// Byte offset of the start of the grapheme cluster in the source text
    pub byte_start: usize,
    /// Byte offset of the end of the grapheme cluster in the source text
    pub byte_end: usize,
}

/// The text shaper, wrapping a `cosmic_text::FontSystem`.
///
/// Provides accurate text measurement and shaping using system fonts
/// and HarfBuzz-based OpenType shaping.
pub struct TextShaper {
    font_system: FontSystem,
}

impl TextShaper {
    /// Create a new text shaper with system font discovery.
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
        }
    }

    /// Build `cosmic_text::Attrs` from font parameters.
    fn build_attrs<'a>(
        family_name: &'a str,
        bold: bool,
        italic: bool,
    ) -> Attrs<'a> {
        let mut attrs = Attrs::new();
        attrs = attrs.family(Family::Name(family_name));
        if bold {
            attrs = attrs.weight(Weight::BOLD);
        }
        if italic {
            attrs = attrs.style(Style::Italic);
        }
        attrs
    }

    /// Measure the dimensions of a text string.
    ///
    /// Uses cosmic_text for accurate, font-aware measurement with
    /// HarfBuzz shaping. The returned width is the total advance width
    /// of all shaped glyphs.
    pub fn measure_text(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
    ) -> TextMetrics {
        if text.is_empty() {
            return TextMetrics {
                width: 0.0,
                height: font_size as f64 * 1.2,
                ascent: font_size as f64 * 0.8,
                descent: font_size as f64 * 0.2,
            };
        }

        let fs = font_size as f32;
        let line_height = fs * 1.2;
        let metrics = Metrics::new(fs, line_height);
        let attrs = Self::build_attrs(font_family, bold, italic);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        // Unbounded width so we get the natural text width
        buffer.set_size(&mut self.font_system, None, None);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut total_width: f64 = 0.0;
        let mut max_height: f64 = 0.0;

        for run in buffer.layout_runs() {
            let mut run_width: f64 = 0.0;
            for glyph in run.glyphs.iter() {
                run_width = (glyph.x + glyph.w) as f64;
            }
            if run_width > total_width {
                total_width = run_width;
            }
            if (run.line_height as f64) > max_height {
                max_height = run.line_height as f64;
            }
        }

        if max_height == 0.0 {
            max_height = line_height as f64;
        }

        let ascent = font_size * 0.8;
        let descent = max_height - ascent;

        TextMetrics {
            width: total_width,
            height: max_height,
            ascent,
            descent: descent.max(0.0),
        }
    }

    /// Measure only the advance width of a text string.
    pub fn measure_width(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
    ) -> f64 {
        self.measure_text(text, font_family, font_size, bold, italic)
            .width
    }

    /// Shape a text string and return individual glyph positions.
    ///
    /// This is used by the layout engine to build `GlyphRun`s with
    /// accurate per-glyph positioning.
    pub fn shape_text(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
    ) -> (Vec<ShapedGlyph>, TextMetrics) {
        if text.is_empty() {
            return (
                Vec::new(),
                TextMetrics {
                    width: 0.0,
                    height: font_size * 1.2,
                    ascent: font_size * 0.8,
                    descent: font_size * 0.2,
                },
            );
        }

        let fs = font_size as f32;
        let line_height = fs * 1.2;
        let metrics = Metrics::new(fs, line_height);
        let attrs = Self::build_attrs(font_family, bold, italic);

        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, None, None);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut shaped_glyphs = Vec::new();
        let mut total_width: f64 = 0.0;
        let mut max_height: f64 = line_height as f64;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                shaped_glyphs.push(ShapedGlyph {
                    glyph_id: glyph.glyph_id as u32,
                    x: glyph.x as f64,
                    advance: glyph.w as f64,
                    byte_start: glyph.start,
                    byte_end: glyph.end,
                });
                let right = (glyph.x + glyph.w) as f64;
                if right > total_width {
                    total_width = right;
                }
            }
            if (run.line_height as f64) > max_height {
                max_height = run.line_height as f64;
            }
        }

        let ascent = font_size * 0.8;
        let descent = (max_height - ascent).max(0.0);

        let text_metrics = TextMetrics {
            width: total_width,
            height: max_height,
            ascent,
            descent,
        };

        (shaped_glyphs, text_metrics)
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure the dimensions of a text string with the given shaping config.
///
/// Convenience function that creates a temporary shaper. For repeated
/// measurements, prefer using `TextShaper` directly to reuse the
/// `FontSystem`.
pub fn measure_text(text: &str, config: &ShapingConfig) -> TextMetrics {
    let mut shaper = TextShaper::new();
    shaper.measure_text(text, &config.default_font, config.default_size, false, false)
}

/// Measure a text string at an explicit font size (ignores config size).
pub fn measure_text_at_size(text: &str, font_size: f64) -> TextMetrics {
    let mut shaper = TextShaper::new();
    shaper.measure_text(text, "Liberation Serif", font_size, false, false)
}
