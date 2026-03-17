//! # rw-track
//!
//! Track changes and comments system for Rust Writer.
//!
//! - Record insertions, deletions, and formatting changes
//! - Accept / reject individual changes or all changes
//! - Filter changes by author or date
//! - Comment / annotation support with replies
//! - Resolve / unresolve comments
//! - Document comparison and merging
//! - Change display modes (all markup, simple markup, no markup, original)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A tracked change in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedChange {
    pub id: Uuid,
    pub change_type: ChangeType,
    pub author: String,
    pub date: DateTime<Utc>,
    /// For text changes: the affected text
    pub text: Option<String>,
    /// For formatting changes: description of the format change
    pub format_description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Insertion,
    Deletion,
    FormatChange,
    TableInsertion,
    TableDeletion,
    StyleChange,
}

/// Display mode for tracked changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupMode {
    /// Show all changes with markup
    AllMarkup,
    /// Show changes with simplified indicators
    SimpleMarkup,
    /// Show document as if all changes were accepted
    NoMarkup,
    /// Show original document before changes
    Original,
}

/// Configuration for the change tracking system.
#[derive(Debug, Clone)]
pub struct TrackingConfig {
    /// Whether tracking is enabled
    pub enabled: bool,
    /// Current display mode
    pub markup_mode: MarkupMode,
    /// Current author name (for new changes)
    pub author: String,
    /// Color assignments for different authors
    pub author_colors: Vec<(String, String)>,
    /// Whether to show formatting changes
    pub show_formatting: bool,
    /// Whether to show insertions
    pub show_insertions: bool,
    /// Whether to show deletions
    pub show_deletions: bool,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            markup_mode: MarkupMode::AllMarkup,
            author: String::new(),
            author_colors: Vec::new(),
            show_formatting: true,
            show_insertions: true,
            show_deletions: true,
        }
    }
}
