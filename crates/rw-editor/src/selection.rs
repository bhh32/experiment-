use crate::cursor::CursorPosition;
use serde::{Deserialize, Serialize};

/// A selection range in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    /// The anchor point (where the selection started)
    pub anchor: CursorPosition,
    /// The active point (where the cursor currently is)
    pub active: CursorPosition,
    /// Selection type
    pub selection_type: SelectionType,
}

impl Selection {
    /// Create a new selection from anchor to active position.
    pub fn new(anchor: CursorPosition, active: CursorPosition) -> Self {
        Self {
            anchor,
            active,
            selection_type: SelectionType::Range,
        }
    }

    /// Get the start (earlier) position of the selection.
    pub fn start(&self) -> &CursorPosition {
        if self.anchor <= self.active {
            &self.anchor
        } else {
            &self.active
        }
    }

    /// Get the end (later) position of the selection.
    pub fn end(&self) -> &CursorPosition {
        if self.anchor <= self.active {
            &self.active
        } else {
            &self.anchor
        }
    }

    /// Whether the selection is empty (anchor == active).
    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    /// Whether the selection is "backwards" (active before anchor).
    pub fn is_reversed(&self) -> bool {
        self.active < self.anchor
    }
}

/// The type of selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionType {
    /// Standard range selection
    Range,
    /// Word selection (double-click)
    Word,
    /// Line selection (triple-click or margin click)
    Line,
    /// Paragraph selection
    Paragraph,
    /// Block/column selection (Alt+drag)
    Block,
    /// Entire document (Ctrl+A)
    All,
}
