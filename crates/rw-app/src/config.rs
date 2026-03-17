//! Application configuration — user preferences, recent files, window state.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

impl AppConfig {
    /// Get the config file path.
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rust-writer").join("config.json"))
    }

    /// Load config from disk, returning defaults if not found.
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// Add a file to the recent files list.
    pub fn add_recent_file(&mut self, path: &str) {
        // Remove if already present
        self.recent_files.retain(|f| f.path != path);

        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        self.recent_files.insert(
            0,
            RecentFile {
                path: path.to_string(),
                name,
                last_opened: chrono::Utc::now().to_rfc3339(),
                pinned: false,
            },
        );

        // Trim to max
        while self.recent_files.len() > self.max_recent {
            self.recent_files.pop();
        }
    }
}
