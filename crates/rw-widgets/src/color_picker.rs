//! Color picker widget.
//!
//! A full-featured color picker with:
//! - Theme colors (from document theme)
//! - Standard colors
//! - Recent colors
//! - Custom color dialog (HSL/RGB/Hex input)

/// Standard colors available in the picker (hex, name).
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
    // Additional standard colors
    ("#800000", "Dark Red"),
    ("#804000", "Brown"),
    ("#808000", "Olive"),
    ("#008000", "Dark Green"),
    ("#008080", "Teal"),
    ("#000080", "Navy"),
    ("#400080", "Dark Purple"),
    ("#800040", "Maroon"),
];

/// Color picker state.
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// Currently selected color (hex string like "#RRGGBB")
    pub selected: Option<String>,
    /// Recently used colors
    pub recent: Vec<String>,
    /// Maximum number of recent colors to remember
    pub max_recent: usize,
    /// Whether the custom color panel is open
    pub custom_panel_open: bool,
    /// Hex string being edited in the custom color input
    pub custom_hex_input: String,
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            selected: None,
            recent: Vec::new(),
            max_recent: 10,
            custom_panel_open: false,
            custom_hex_input: String::new(),
        }
    }
}

impl ColorPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a color, adding it to the recent list.
    pub fn select(&mut self, hex: impl Into<String>) {
        let color = hex.into();
        self.selected = Some(color.clone());
        self.push_recent(color);
    }

    /// Add a color to the recent list, removing the oldest if at capacity.
    pub fn push_recent(&mut self, hex: String) {
        // Deduplicate
        self.recent.retain(|c| c != &hex);
        self.recent.insert(0, hex);
        if self.recent.len() > self.max_recent {
            self.recent.truncate(self.max_recent);
        }
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }
}

/// Parse a hex color string like "#RRGGBB" or "RRGGBB" into (r, g, b) components.
pub fn custom_color_from_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else if hex.len() == 3 {
        // Short form: #RGB -> #RRGGBB
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

/// Convert (r, g, b) to a hex string "#RRGGBB".
pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Convert (r, g, b) to HSL (h: 0-360, s: 0-1, l: 0-1).
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * ((bf - rf) / delta + 2.0)
    } else {
        60.0 * ((rf - gf) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, l)
}
