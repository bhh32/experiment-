//! Cross-reference fields.

/// What to display for a cross-reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossRefDisplay {
    /// Show the referenced text content
    Text,
    /// Show the page number
    PageNumber,
    /// Show "above" or "below" relative to current position
    AboveBelow,
    /// Show the paragraph number (for numbered paragraphs)
    ParagraphNumber,
    /// Show the heading number
    HeadingNumber,
    /// Show the full context (e.g., "Figure 3")
    NumberedCaption,
    /// Show only the caption text
    CaptionText,
}
