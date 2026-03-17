//! Application configuration — user preferences, recent files, window state.

use serde::{Deserialize, Serialize};

/// Persistent application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Recent files list
    pub recent_files: Vec<RecentFile>,
    /// Maximum recent files to remember
    pub max_recent: usize,
    /// Default save format
    pub default_format: String,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,
    /// Show ruler
    pub show_ruler: bool,
    /// Show formatting marks
    pub show_formatting_marks: bool,
    /// Default zoom percentage
    pub default_zoom: u32,
    /// Spell check enabled
    pub spell_check: bool,
    /// Default language
    pub language: String,
    /// Theme preference: "system", "light", "dark"
    pub theme: String,
    /// Measurement unit: "inches", "cm", "mm", "pt"
    pub measurement_unit: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            max_recent: 20,
            default_format: "odt".to_string(),
            auto_save_interval: 300,
            show_ruler: true,
            show_formatting_marks: false,
            default_zoom: 100,
            spell_check: true,
            language: "en_US".to_string(),
            theme: "system".to_string(),
            measurement_unit: "inches".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub last_opened: String,
    pub pinned: bool,
}
