//! Status bar widget — the bar at the bottom of the window.

/// Status bar configuration — which items to show.
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    pub show_page_number: bool,
    pub show_section: bool,
    pub show_line_number: bool,
    pub show_column_number: bool,
    pub show_word_count: bool,
    pub show_language: bool,
    pub show_insert_mode: bool,
    pub show_spell_check: bool,
    pub show_track_changes: bool,
    pub show_zoom_slider: bool,
    pub show_view_buttons: bool,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            show_page_number: true,
            show_section: false,
            show_line_number: true,
            show_column_number: true,
            show_word_count: true,
            show_language: true,
            show_insert_mode: true,
            show_spell_check: true,
            show_track_changes: true,
            show_zoom_slider: true,
            show_view_buttons: true,
        }
    }
}

/// Current status bar state.
#[derive(Debug, Clone, Default)]
pub struct StatusBarState {
    pub page: usize,
    pub total_pages: usize,
    pub section: usize,
    pub line: usize,
    pub column: usize,
    pub word_count: usize,
    pub char_count: usize,
    pub language: String,
    pub insert_mode: bool,
    pub spell_check_ok: bool,
    pub track_changes_on: bool,
    pub zoom_percent: u32,
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    PrintLayout,
    WebLayout,
    Outline,
    Draft,
    ReadMode,
    FocusMode,
}
