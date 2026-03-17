//! # rw-find
//!
//! Find and replace engine for Rust Writer.
//!
//! - Plain text search (case-sensitive and case-insensitive)
//! - Whole word matching
//! - Regular expression search (with capture groups)
//! - Search within selection
//! - Search by formatting (find all bold text, find by style, etc.)
//! - Replace with captured groups ($1, $2, etc.)
//! - Replace all / replace one at a time
//! - Find next / find previous
//! - Match highlighting

/// Search options.
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// The search pattern
    pub pattern: String,
    /// Whether to use regex
    pub use_regex: bool,
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Match whole words only
    pub whole_word: bool,
    /// Search direction
    pub direction: SearchDirection,
    /// Whether to wrap around at document end
    pub wrap_around: bool,
    /// Search within selection only
    pub in_selection: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            use_regex: false,
            case_sensitive: false,
            whole_word: false,
            direction: SearchDirection::Forward,
            wrap_around: true,
            in_selection: false,
        }
    }
}

/// Replace options (extends FindOptions).
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub find: FindOptions,
    /// The replacement string (supports $1, $2 for regex captures)
    pub replacement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// A search match result.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Section index
    pub section: usize,
    /// Block index
    pub block: usize,
    /// Start byte offset within the paragraph text
    pub start_offset: usize,
    /// End byte offset
    pub end_offset: usize,
    /// The matched text
    pub matched_text: String,
}
