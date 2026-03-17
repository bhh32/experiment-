//! Table of contents generation.

/// Configuration for generating a table of contents.
#[derive(Debug, Clone)]
pub struct TocConfig {
    /// Minimum heading level to include (1 = Heading 1)
    pub from_level: u8,
    /// Maximum heading level to include
    pub to_level: u8,
    /// Show page numbers
    pub show_page_numbers: bool,
    /// Right-align page numbers
    pub right_align_numbers: bool,
    /// Leader character between text and page number
    pub leader: TocLeader,
    /// Use hyperlinks
    pub use_hyperlinks: bool,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            from_level: 1,
            to_level: 3,
            show_page_numbers: true,
            right_align_numbers: true,
            leader: TocLeader::Dot,
            use_hyperlinks: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TocLeader {
    None,
    Dot,
    Dash,
    Underscore,
}
