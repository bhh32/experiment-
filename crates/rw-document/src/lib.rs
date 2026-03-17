//! # rw-document
//!
//! Core document model for Rust Writer.
//!
//! This crate defines the Document Object Model (DOM) that represents
//! the complete structure of a word processing document. It is the
//! central data structure that all other crates operate on.
//!
//! ## Architecture
//!
//! The document model is a tree structure:
//!
//! ```text
//! Document
//! ├── Metadata (title, author, dates, custom properties)
//! ├── Sections[]
//! │   ├── SectionProperties (columns, page style, margins)
//! │   └── Block[]
//! │       ├── Paragraph
//! │       │   ├── ParagraphProperties (style, alignment, spacing, indent)
//! │       │   └── Inline[]
//! │       │       ├── TextRun { text, CharacterProperties }
//! │       │       ├── InlineImage { image_ref, size, ... }
//! │       │       ├── Field { field_type, ... }
//! │       │       ├── FootnoteRef { note_id }
//! │       │       ├── BookmarkStart / BookmarkEnd
//! │       │       ├── CommentRangeStart / CommentRangeEnd
//! │       │       └── Break { break_type }
//! │       ├── Table
//! │       │   └── (defined in rw-tables, referenced here)
//! │       ├── Drawing
//! │       │   └── (defined in rw-media, referenced here)
//! │       └── HorizontalRule
//! ├── Headers / Footers
//! ├── Footnotes / Endnotes
//! ├── Comments
//! └── Styles (delegated to rw-styles)
//! ```

pub mod block;
pub mod document;
pub mod inline;
pub mod metadata;
pub mod paragraph;
pub mod properties;
pub mod section;
pub mod text_run;

pub use document::Document;
pub use metadata::Metadata;
pub use section::Section;
pub use paragraph::Paragraph;
pub use text_run::TextRun;
pub use block::Block;
pub use inline::Inline;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for any element in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(Uuid);

impl ElementId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// Units of measurement used throughout the document model.
/// Internally, all measurements are stored in twips (1/1440 of an inch).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Twips(pub i32);

impl Twips {
    pub const ZERO: Self = Self(0);

    /// Create from inches
    pub fn from_inches(inches: f64) -> Self {
        Self((inches * 1440.0) as i32)
    }

    /// Create from centimeters
    pub fn from_cm(cm: f64) -> Self {
        Self((cm * 567.0) as i32)
    }

    /// Create from points (1/72 inch)
    pub fn from_points(points: f64) -> Self {
        Self((points * 20.0) as i32)
    }

    /// Create from millimeters
    pub fn from_mm(mm: f64) -> Self {
        Self((mm * 56.7) as i32)
    }

    /// Convert to inches
    pub fn to_inches(self) -> f64 {
        self.0 as f64 / 1440.0
    }

    /// Convert to points
    pub fn to_points(self) -> f64 {
        self.0 as f64 / 20.0
    }

    /// Convert to centimeters
    pub fn to_cm(self) -> f64 {
        self.0 as f64 / 567.0
    }

    /// Convert to pixels at a given DPI
    pub fn to_pixels(self, dpi: f64) -> f64 {
        self.to_inches() * dpi
    }
}

/// A color represented as RGBA (0-255 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create from a hex string like "#FF0000" or "FF0000"
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Self::rgba(r, g, b, a))
        } else {
            None
        }
    }
}
