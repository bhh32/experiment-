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

use std::collections::HashMap;
use rw_document::inline::ImageSource;
use rw_document::Twips;

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

/// A media resource tracked by the MediaManager.
#[derive(Debug, Clone)]
pub struct MediaResource {
    pub id: String,
    pub source: ImageSource,
    pub format: Option<ImageFormat>,
    /// Natural width in twips
    pub natural_width: Twips,
    /// Natural height in twips
    pub natural_height: Twips,
}

/// Central manager for embedded media resources.
///
/// Tracks all images and other media embedded in or linked from the document.
#[derive(Debug, Default)]
pub struct MediaManager {
    resources: HashMap<String, MediaResource>,
}

impl MediaManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new media resource and return its ID.
    pub fn add_resource(&mut self, resource: MediaResource) -> String {
        let id = resource.id.clone();
        self.resources.insert(id.clone(), resource);
        id
    }

    /// Look up a resource by ID.
    pub fn get_resource(&self, id: &str) -> Option<&MediaResource> {
        self.resources.get(id)
    }

    /// Remove a resource by ID.
    pub fn remove_resource(&mut self, id: &str) -> Option<MediaResource> {
        self.resources.remove(id)
    }

    /// List all resource IDs.
    pub fn list_resources(&self) -> Vec<&str> {
        self.resources.keys().map(|s| s.as_str()).collect()
    }

    /// Update the natural dimensions of a resource (e.g. after loading).
    pub fn update_dimensions(&mut self, id: &str, width: Twips, height: Twips) {
        if let Some(r) = self.resources.get_mut(id) {
            r.natural_width = width;
            r.natural_height = height;
        }
    }
}
