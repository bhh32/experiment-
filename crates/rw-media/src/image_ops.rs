//! Image operations — loading, resizing, cropping, format conversion.

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
