use serde::{Deserialize, Serialize};

/// A document theme — a coordinated set of colors, fonts, and effects.
///
/// Themes are inspired by Microsoft Word's theme system and allow
/// documents to have a consistent visual identity that can be
/// switched without manually changing each style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            colors: ThemeColors::default(),
            fonts: ThemeFonts::default(),
        }
    }
}

/// Theme color palette.
/// Based on the OOXML theme color model with 12 semantic color slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Primary dark color (typically used for text)
    pub dark1: String,
    /// Secondary dark color
    pub dark2: String,
    /// Primary light color (typically used for backgrounds)
    pub light1: String,
    /// Secondary light color
    pub light2: String,
    /// Accent colors (1-6)
    pub accent1: String,
    pub accent2: String,
    pub accent3: String,
    pub accent4: String,
    pub accent5: String,
    pub accent6: String,
    /// Hyperlink color
    pub hyperlink: String,
    /// Followed hyperlink color
    pub followed_hyperlink: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            dark1: "#000000".to_string(),
            dark2: "#44546A".to_string(),
            light1: "#FFFFFF".to_string(),
            light2: "#E7E6E6".to_string(),
            accent1: "#4472C4".to_string(),
            accent2: "#ED7D31".to_string(),
            accent3: "#A5A5A5".to_string(),
            accent4: "#FFC000".to_string(),
            accent5: "#5B9BD5".to_string(),
            accent6: "#70AD47".to_string(),
            hyperlink: "#0563C1".to_string(),
            followed_hyperlink: "#954F72".to_string(),
        }
    }
}

/// Theme font definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    /// Font for headings
    pub heading: String,
    /// Font for body text
    pub body: String,
    /// Monospace font (for code)
    pub monospace: String,
}

impl Default for ThemeFonts {
    fn default() -> Self {
        Self {
            heading: "Calibri".to_string(),
            body: "Calibri".to_string(),
            monospace: "Consolas".to_string(),
        }
    }
}
