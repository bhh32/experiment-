//! Application message types.
//!
//! All user interactions and events are represented as messages
//! following the Elm Architecture (TEA / MVU) pattern.

use rw_editor::operations::{CharacterFormatOp, DeleteDirection, ListType, ParagraphFormatOp};
use rw_ribbon::TabId;
use rw_widgets::status_bar::ViewMode;

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// No-op message
    None,

    // --- File operations ---
    /// Create a new empty document
    NewDocument,
    /// Open a file (path provided by file dialog)
    OpenFile(String),
    /// Save the current document
    Save,
    /// Save with a new path
    SaveAs(String),
    /// Export to a specific format
    Export(ExportFormat, String),
    /// Close the current document
    CloseDocument,

    // --- Edit operations ---
    /// Insert text at cursor
    InsertText(String),
    /// Delete in direction
    Delete(DeleteDirection),
    /// Insert paragraph break (Enter)
    InsertParagraphBreak,
    /// Insert line break (Shift+Enter)
    InsertLineBreak,
    /// Insert page break (Ctrl+Enter)
    InsertPageBreak,
    /// Insert tab
    InsertTab,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Cut selection
    Cut,
    /// Copy selection
    Copy,
    /// Paste clipboard content
    Paste(String),
    /// Select all
    SelectAll,

    // --- Formatting ---
    /// Apply character formatting
    CharacterFormat(CharacterFormatOp),
    /// Apply paragraph formatting
    ParagraphFormat(ParagraphFormatOp),
    /// Apply a named paragraph style
    ApplyStyle(String),
    /// Toggle list
    ToggleList(ListType),
    /// Increase indent
    IncreaseIndent,
    /// Decrease indent
    DecreaseIndent,
    /// Clear formatting
    ClearFormatting,

    // --- Cursor / Selection ---
    /// Move cursor (direction_key, extend_selection)
    CursorMove(CursorMoveMsg),

    // --- Find/Replace ---
    /// Open find dialog
    OpenFind,
    /// Open replace dialog
    OpenReplace,
    /// Find next occurrence
    FindNext(String),
    /// Find previous occurrence
    FindPrev(String),
    /// Replace current match
    ReplaceNext(String, String),
    /// Replace all matches
    ReplaceAll(String, String),

    // --- Ribbon ---
    /// Switch the active ribbon tab
    TabChanged(TabId),
    /// Ribbon item clicked
    RibbonAction(String),

    // --- View ---
    /// Change zoom level
    ZoomChanged(u32),
    /// Toggle focus/zen mode
    ToggleFocusMode,
    /// Toggle command palette
    ToggleCommandPalette,
    /// Change view mode
    ViewModeChanged(ViewMode),
    /// Toggle ruler visibility
    ToggleRuler,
    /// Toggle formatting marks visibility
    ToggleFormattingMarks,
    /// Toggle navigation pane
    ToggleNavigationPane,

    // --- Track changes ---
    /// Toggle track changes
    ToggleTrackChanges,
    /// Accept current change
    AcceptChange,
    /// Reject current change
    RejectChange,
    /// Accept all changes
    AcceptAllChanges,
    /// Reject all changes
    RejectAllChanges,

    // --- Insert ---
    /// Insert a table with (rows, cols)
    InsertTable(usize, usize),
    /// Insert image from path
    InsertImage(String),
    /// Insert hyperlink
    InsertHyperlink(String, String),
    /// Insert bookmark
    InsertBookmark(String),
    /// Insert page number field
    InsertPageNumber,
    /// Insert date field
    InsertDate,

    // --- Print ---
    /// Print the document
    Print,
    /// Print preview
    PrintPreview,

    // --- Spell check ---
    /// Toggle spell checking
    ToggleSpellCheck,

    // --- Window ---
    /// Window resize event
    WindowResized(u32, u32),

    // --- Command palette ---
    /// Command palette query changed
    PaletteQueryChanged(String),
    /// Command palette item selected
    PaletteItemSelected(String),
}

/// Cursor movement message.
#[derive(Debug, Clone)]
pub struct CursorMoveMsg {
    pub direction: CursorDirection,
    pub unit: CursorUnit,
    pub extend_selection: bool,
}

/// Cursor movement direction.
#[derive(Debug, Clone, Copy)]
pub enum CursorDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Cursor movement unit.
#[derive(Debug, Clone, Copy)]
pub enum CursorUnit {
    Character,
    Word,
    Line,
    Paragraph,
    Page,
    Document,
}

/// Export format.
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Pdf,
    Html,
    Rtf,
    PlainText,
    Docx,
    Odt,
    Epub,
}
