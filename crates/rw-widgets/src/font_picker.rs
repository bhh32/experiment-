//! Font picker widget — dropdown with font preview.

/// Well-known font families available on most systems.
pub const COMMON_FONTS: &[&str] = &[
    "Arial",
    "Arial Black",
    "Calibri",
    "Cambria",
    "Comic Sans MS",
    "Consolas",
    "Constantia",
    "Corbel",
    "Courier New",
    "DejaVu Sans",
    "DejaVu Serif",
    "FreeMono",
    "FreeSans",
    "FreeSerif",
    "Georgia",
    "Impact",
    "Liberation Mono",
    "Liberation Sans",
    "Liberation Serif",
    "Linux Libertine",
    "Lucida Console",
    "Palatino Linotype",
    "Segoe UI",
    "Tahoma",
    "Times New Roman",
    "Trebuchet MS",
    "Ubuntu",
    "Ubuntu Mono",
    "Verdana",
];

/// Return the list of common font names.
///
/// In a real implementation this would enumerate the installed system fonts.
/// For now it returns the static list of commonly available fonts.
pub fn get_system_fonts() -> Vec<String> {
    COMMON_FONTS.iter().map(|s| s.to_string()).collect()
}

/// Font categories for organising the picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontCategory {
    /// Recently used fonts
    Recent,
    /// Theme / document fonts
    Theme,
    /// All other installed fonts
    All,
}

/// Font picker widget state.
#[derive(Debug, Clone)]
pub struct FontPickerState {
    /// The full list of available font names
    pub fonts: Vec<String>,
    /// Currently selected font
    pub selected: Option<String>,
    /// Filter / search string
    pub filter: String,
    /// Recently used fonts (most recent first)
    pub recent: Vec<String>,
    /// Maximum recent fonts to remember
    pub max_recent: usize,
    /// Whether the dropdown is open
    pub open: bool,
}

impl Default for FontPickerState {
    fn default() -> Self {
        Self {
            fonts: get_system_fonts(),
            selected: Some("Calibri".to_string()),
            filter: String::new(),
            recent: Vec::new(),
            max_recent: 5,
            open: false,
        }
    }
}

impl FontPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a font by name, adding it to the recent list.
    pub fn select(&mut self, font: impl Into<String>) {
        let font = font.into();
        self.selected = Some(font.clone());
        self.push_recent(font);
        self.filter.clear();
        self.open = false;
    }

    /// Add a font to the recent list.
    fn push_recent(&mut self, font: String) {
        self.recent.retain(|f| f != &font);
        self.recent.insert(0, font);
        if self.recent.len() > self.max_recent {
            self.recent.truncate(self.max_recent);
        }
    }

    /// Return fonts that match the current filter string (case-insensitive).
    pub fn filtered_fonts(&self) -> Vec<&str> {
        if self.filter.is_empty() {
            self.fonts.iter().map(|s| s.as_str()).collect()
        } else {
            let lc = self.filter.to_lowercase();
            self.fonts
                .iter()
                .filter(|f| f.to_lowercase().contains(&lc))
                .map(|s| s.as_str())
                .collect()
        }
    }
}
