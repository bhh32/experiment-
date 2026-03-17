//! Application message types.
//!
//! All user interactions and events are represented as messages
//! following the Elm Architecture (TEA / MVU) pattern.

use rw_editor::operations::{CharacterFormatOp, DeleteDirection, ListType, ParagraphFormatOp};
use rw_ribbon::TabId;
use rw_widgets::status_bar::ViewMode;

/// Top-level application messages.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    /// No-op message
    None,

    // --- File operations ---
    /// Create a new empty document
    NewDocument,
    /// Open file dialog requested
    RequestOpenFile,
    /// Open a file (path provided by file dialog)
    OpenFile(String),
    /// Save the current document
    Save,
    /// Request a save-as dialog
    RequestSaveAs,
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
    /// Close find/replace bar
    CloseFindReplace,

    // --- Ribbon ---
    /// Switch the active ribbon tab
    TabChanged(TabId),
    /// Ribbon item clicked
    RibbonAction(String),

    // --- View ---
    /// Change zoom level
    ZoomChanged(u32),
    /// Zoom in
    ZoomIn,
    /// Zoom out
    ZoomOut,
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
    /// Insert horizontal rule
    InsertHorizontalRule,
    /// Insert symbol/special character
    InsertSymbol(char),

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

    // --- Backstage ---
    /// Open the backstage/File view
    OpenBackstage,
    /// Close the backstage/File view
    CloseBackstage,
    /// Navigate to a backstage page
    BackstageNavigate(BackstagePage),

    // --- Keyboard event ---
    /// Raw key press event from subscription
    KeyPressed(KeyCode, KeyModifiers),
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum ExportFormat {
    Pdf,
    Html,
    Rtf,
    PlainText,
    Docx,
    Odt,
    Epub,
}

/// Backstage page identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackstagePage {
    Info,
    New,
    Open,
    Recent,
    Save,
    SaveAs,
    Print,
    Export,
    Options,
}

/// Keyboard key codes (simplified from iced).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Character(char),
    Enter,
    Tab,
    Backspace,
    Delete,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Keyboard modifiers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}
