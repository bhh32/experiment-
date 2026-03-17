//! Document editing operations.
//!
//! Each operation is a high-level action that modifies the document
//! through the undo system. Operations are the bridge between user
//! input (keystrokes, menu actions) and document model mutations.

/// High-level editing operations that can be triggered by the UI.
#[derive(Debug, Clone)]
pub enum EditOperation {
    /// Insert text at the current cursor position
    InsertText(String),
    /// Delete text in the given direction
    Delete(DeleteDirection),
    /// Insert a paragraph break (Enter)
    InsertParagraphBreak,
    /// Insert a line break (Shift+Enter)
    InsertLineBreak,
    /// Insert a page break (Ctrl+Enter)
    InsertPageBreak,
    /// Insert a tab character
    InsertTab,
    /// Apply character formatting to the selection
    ApplyCharacterFormat(CharacterFormatOp),
    /// Apply paragraph formatting
    ApplyParagraphFormat(ParagraphFormatOp),
    /// Apply a paragraph style
    ApplyParagraphStyle(String),
    /// Apply a character style
    ApplyCharacterStyle(String),
    /// Toggle a list style on the current paragraph(s)
    ToggleList(ListType),
    /// Increase indentation
    IncreaseIndent,
    /// Decrease indentation
    DecreaseIndent,
    /// Clear all direct formatting (reset to style)
    ClearFormatting,
}

#[derive(Debug, Clone, Copy)]
pub enum DeleteDirection {
    /// Delete the character/selection before the cursor (Backspace)
    Backward,
    /// Delete the character/selection after the cursor (Delete)
    Forward,
    /// Delete the word before the cursor (Ctrl+Backspace)
    WordBackward,
    /// Delete the word after the cursor (Ctrl+Delete)
    WordForward,
}

#[derive(Debug, Clone)]
pub enum CharacterFormatOp {
    ToggleBold,
    ToggleItalic,
    ToggleUnderline,
    ToggleStrikethrough,
    ToggleSuperscript,
    ToggleSubscript,
    SetFontFamily(String),
    SetFontSize(u32), // in half-points
    SetColor(String),
    SetHighlight(String),
}

#[derive(Debug, Clone)]
pub enum ParagraphFormatOp {
    SetAlignment(String),
    SetLineSpacing(f64),
    SetSpaceBefore(i32), // twips
    SetSpaceAfter(i32),  // twips
    SetIndentLeft(i32),
    SetIndentRight(i32),
    SetFirstLineIndent(i32),
}

#[derive(Debug, Clone, Copy)]
pub enum ListType {
    Bullet,
    Numbered,
}
