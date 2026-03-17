//! # rw-styles
//!
//! The style system for Rust Writer.
//!
//! Implements all six style categories found in professional word processors:
//! - **Paragraph styles**: Font, size, alignment, spacing, indent, tabs, borders, outline level
//! - **Character styles**: Font overrides applied to text spans within paragraphs
//! - **Page styles**: Page size, margins, orientation, columns, headers/footers
//! - **List styles**: Numbering/bullet definitions for up to 10 outline levels
//! - **Table styles**: Border, shading, and text formatting for table cells
//! - **Frame styles**: Properties for text frames, image frames, OLE objects
//!
//! ## Style Inheritance
//!
//! Styles support single inheritance via the `parent` field. Properties
//! cascade from parent to child — a child style only needs to specify
//! the properties it overrides. The resolution order is:
//!
//! 1. Direct formatting (applied by the user)
//! 2. Character style (if any)
//! 3. Paragraph style
//! 4. Parent paragraph style (recursive)
//! 5. Document defaults

pub mod catalog;
pub mod character;
pub mod defaults;
pub mod frame;
pub mod list;
pub mod page;
pub mod paragraph;
pub mod table;
pub mod theme;

pub use catalog::StyleCatalog;
pub use theme::Theme;

use serde::{Deserialize, Serialize};

/// Unique name identifying a style within its category.
pub type StyleName = String;

/// The category of a style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StyleCategory {
    Paragraph,
    Character,
    Page,
    List,
    Table,
    Frame,
}

/// Common properties shared by all style types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleBase {
    /// The unique name of this style
    pub name: StyleName,
    /// Display name (for UI)
    pub display_name: Option<String>,
    /// Parent style name (for inheritance)
    pub parent: Option<StyleName>,
    /// Next style (auto-applied after pressing Enter)
    pub next_style: Option<StyleName>,
    /// Whether this is a built-in default style
    pub built_in: bool,
    /// Whether this style is hidden from the UI
    pub hidden: bool,
    /// Whether this style auto-updates from direct formatting
    pub auto_update: bool,
    /// Style category
    pub category: StyleCategory,
}

impl StyleBase {
    pub fn new(name: impl Into<String>, category: StyleCategory) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            parent: None,
            next_style: None,
            built_in: false,
            hidden: false,
            auto_update: false,
            category,
        }
    }
}
