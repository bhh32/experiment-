//! # rw-editor
//!
//! The editing engine for Rust Writer.
//!
//! This crate handles all user editing operations on the document model:
//! - Cursor positioning and movement (by character, word, line, paragraph, page)
//! - Text selection (point, range, block/column selection)
//! - Text input and deletion
//! - Clipboard operations (cut, copy, paste)
//! - Formatting application (bold, italic, font changes, etc.)
//! - Paragraph operations (split, merge, style changes)
//! - Integration with the undo/redo system
//!
//! The editor does NOT handle rendering or UI — it operates purely on
//! the document model and emits commands for the undo system.

pub mod cursor;
pub mod selection;
pub mod operations;
pub mod input;
pub mod clipboard;

pub use cursor::{Cursor, CursorPosition};
pub use selection::Selection;

use rw_document::Document;
use rw_undo::UndoManager;

/// The editor state — wraps a document with editing state.
pub struct EditorState {
    /// The document being edited
    pub document: Document,
    /// Current cursor position
    pub cursor: Cursor,
    /// Current selection (if any)
    pub selection: Option<Selection>,
    /// Undo/redo manager
    pub undo_manager: UndoManager<Document>,
    /// Whether track changes is enabled
    pub track_changes: bool,
    /// Whether the document is in read-only mode
    pub read_only: bool,
    /// The current insert/overwrite mode
    pub overwrite_mode: bool,
}

impl EditorState {
    /// Create a new editor with an empty document.
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor: Cursor::default(),
            selection: None,
            undo_manager: UndoManager::new(),
            track_changes: false,
            read_only: false,
            overwrite_mode: false,
        }
    }

    /// Create a new editor with an existing document.
    pub fn with_document(document: Document) -> Self {
        Self {
            document,
            cursor: Cursor::default(),
            selection: None,
            undo_manager: UndoManager::new(),
            track_changes: false,
            read_only: false,
            overwrite_mode: false,
        }
    }

    /// Whether the document has unsaved modifications.
    pub fn is_dirty(&self) -> bool {
        self.undo_manager.is_dirty()
    }

    /// Mark the document as saved.
    pub fn mark_saved(&mut self) {
        self.undo_manager.mark_saved();
    }

    /// Check if there's an active selection.
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
