//! Zoom slider widget — zoom control for the document canvas.

/// Predefined zoom levels.
pub const ZOOM_PRESETS: &[u32] = &[25, 50, 75, 100, 125, 150, 200, 300, 400, 500];

/// Zoom slider state.
#[derive(Debug, Clone)]
pub struct ZoomState {
    /// Current zoom percentage
    pub percent: u32,
    /// Minimum zoom
    pub min: u32,
    /// Maximum zoom
    pub max: u32,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            percent: 100,
            min: 10,
            max: 500,
        }
    }
}
