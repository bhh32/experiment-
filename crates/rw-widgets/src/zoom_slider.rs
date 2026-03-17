//! Zoom slider widget — zoom control for the document canvas.

/// Predefined zoom levels.
pub const ZOOM_PRESETS: &[u32] = &[25, 50, 75, 100, 125, 150, 200, 300, 400, 500];

/// Zoom slider state.
#[derive(Debug, Clone)]
pub struct ZoomState {
    /// Current zoom percentage (e.g., 100 = 100%)
    pub percent: u32,
    /// Minimum zoom percentage
    pub min: u32,
    /// Maximum zoom percentage
    pub max: u32,
    /// Preset zoom levels to offer in the dropdown
    pub presets: Vec<u32>,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            percent: 100,
            min: 10,
            max: 500,
            presets: ZOOM_PRESETS.to_vec(),
        }
    }
}

impl ZoomState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the zoom as a floating-point multiplier (1.0 = 100%).
    pub fn factor(&self) -> f64 {
        self.percent as f64 / 100.0
    }

    /// Increase zoom to the next preset level.
    pub fn zoom_in(&mut self) {
        if let Some(&next) = self.presets.iter().find(|&&p| p > self.percent) {
            self.percent = next.min(self.max);
        } else {
            // Already at or above highest preset — step by 10
            self.percent = (self.percent + 10).min(self.max);
        }
    }

    /// Decrease zoom to the previous preset level.
    pub fn zoom_out(&mut self) {
        if let Some(&prev) = self.presets.iter().rev().find(|&&p| p < self.percent) {
            self.percent = prev.max(self.min);
        } else {
            // Already at or below lowest preset — step by 10
            self.percent = self.percent.saturating_sub(10).max(self.min);
        }
    }

    /// Set a specific zoom percentage, clamped to [min, max].
    pub fn set_percent(&mut self, percent: u32) {
        self.percent = percent.clamp(self.min, self.max);
    }

    /// Snap to the closest preset zoom level.
    pub fn snap_to_preset(&mut self) {
        if let Some(&closest) = self.presets.iter().min_by_key(|&&p| {
            let diff = p as i32 - self.percent as i32;
            diff.unsigned_abs()
        }) {
            self.percent = closest;
        }
    }

    /// Zoom to fit the page in the viewport.
    ///
    /// `page_width` and `viewport_width` should be in the same units.
    pub fn zoom_to_fit(&mut self, page_width: f64, viewport_width: f64) {
        if page_width > 0.0 {
            let fit_percent = ((viewport_width / page_width) * 100.0) as u32;
            self.set_percent(fit_percent);
        }
    }

    /// Zoom to fit the page width in the viewport.
    pub fn zoom_to_page_width(&mut self, page_width: f64, viewport_width: f64) {
        self.zoom_to_fit(page_width, viewport_width);
    }
}
