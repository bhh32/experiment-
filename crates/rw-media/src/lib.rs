//! # rw-media
//!
//! Image and media handling for Rust Writer.
//!
//! - Image import (PNG, JPEG, GIF, BMP, TIFF, WebP, SVG)
//! - Image resizing, cropping, and positioning
//! - Text wrapping around images (square, tight, through, top-bottom)
//! - Anchoring (to page, paragraph, character, inline)
//! - Drawing shapes (rectangles, ovals, lines, arrows, callouts)
//! - Shape formatting (fill, outline, shadow, 3D effects)
//! - Image compression and optimization

pub mod image_ops;
pub mod shapes;
pub mod wrapping;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Tiff,
    WebP,
    Svg,
}

impl ImageFormat {
    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            "webp" => Some(Self::WebP),
            "svg" => Some(Self::Svg),
            _ => None,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::WebP => "image/webp",
            Self::Svg => "image/svg+xml",
        }
    }
}
