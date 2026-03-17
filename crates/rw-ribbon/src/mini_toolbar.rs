//! Mini toolbar — the floating formatting toolbar that appears on text selection.
//!
//! Shows commonly used formatting options (font, size, bold, italic, color)
//! in a compact floating toolbar above the selection.

/// Items available in the mini toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniToolbarItem {
    FontFamily,
    FontSize,
    IncreaseFontSize,
    DecreaseFontSize,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Subscript,
    Superscript,
    FontColor,
    HighlightColor,
    BulletList,
    NumberedList,
    AlignLeft,
    AlignCenter,
    AlignRight,
    IndentDecrease,
    IndentIncrease,
    StylesGallery,
}
