//! Image operations — loading, resizing, cropping, format conversion.
//!
//! These operations work on image *metadata* (dimensions, crop rectangles,
//! rotation flags) rather than pixel data. Actual pixel manipulation would
//! be performed by the `image` crate at render/export time.

use rw_document::inline::InlineImage;
use rw_document::Twips;

/// Image dimensions in pixels.
#[derive(Debug, Clone, Copy)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Crop region for an image.
#[derive(Debug, Clone, Copy)]
pub struct CropRegion {
    /// Percentage cropped from the left (0.0 - 1.0)
    pub left: f64,
    /// Percentage cropped from the top
    pub top: f64,
    /// Percentage cropped from the right
    pub right: f64,
    /// Percentage cropped from the bottom
    pub bottom: f64,
}

impl Default for CropRegion {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

/// Image adjustments.
#[derive(Debug, Clone, Copy)]
pub struct ImageAdjustments {
    /// Brightness (-100 to 100)
    pub brightness: i8,
    /// Contrast (-100 to 100)
    pub contrast: i8,
    /// Rotation in degrees (0, 90, 180, 270)
    pub rotation: u16,
    /// Horizontal flip
    pub flip_horizontal: bool,
    /// Vertical flip
    pub flip_vertical: bool,
    /// Transparency (0.0 = opaque, 1.0 = fully transparent)
    pub transparency: f64,
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            brightness: 0,
            contrast: 0,
            rotation: 0,
            flip_horizontal: false,
            flip_vertical: false,
            transparency: 0.0,
        }
    }
}

/// Resize an inline image to the given display dimensions (in twips).
///
/// This updates the `width` and `height` fields on the `InlineImage`.
/// Pixel data is not modified.
pub fn resize_image(image: &mut InlineImage, new_width: Twips, new_height: Twips) {
    image.width = new_width;
    image.height = new_height;
}

/// Resize an image while maintaining its aspect ratio.
///
/// `max_width` and `max_height` are the bounding box in twips.
/// The image is scaled to fit within the box while preserving ratio.
pub fn resize_image_fit(
    image: &mut InlineImage,
    max_width: Twips,
    max_height: Twips,
) {
    let orig_w = image.width.0 as f64;
    let orig_h = image.height.0 as f64;
    let max_w = max_width.0 as f64;
    let max_h = max_height.0 as f64;

    if orig_w <= 0.0 || orig_h <= 0.0 {
        return;
    }

    let scale = (max_w / orig_w).min(max_h / orig_h);
    image.width = Twips((orig_w * scale) as i32);
    image.height = Twips((orig_h * scale) as i32);
}

/// Apply a crop region to an inline image (metadata only).
///
/// The crop is stored by adjusting the display dimensions to reflect the
/// visible area after cropping.
pub fn crop_image(image: &mut InlineImage, crop: &CropRegion) {
    let w = image.width.0 as f64;
    let h = image.height.0 as f64;

    let visible_w_frac = 1.0 - crop.left - crop.right;
    let visible_h_frac = 1.0 - crop.top - crop.bottom;

    // Clamp to at least 1 twip to avoid zero-size
    image.width = Twips(((w * visible_w_frac) as i32).max(1));
    image.height = Twips(((h * visible_h_frac) as i32).max(1));
}

/// Rotate an inline image by swapping width/height for 90° or 270° rotations.
///
/// `degrees` must be 0, 90, 180, or 270. Other values are ignored.
pub fn rotate_image(image: &mut InlineImage, degrees: u16) {
    match degrees % 360 {
        90 | 270 => {
            // Swap width and height
            std::mem::swap(&mut image.width, &mut image.height);
        }
        180 | 0 => {
            // No dimension change for 180° or 0°
        }
        _ => {} // Non-standard angle — do nothing
    }
}
