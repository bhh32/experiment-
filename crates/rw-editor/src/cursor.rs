use serde::{Deserialize, Serialize};

/// A position within the document, identified by section, block, and
/// inline offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Section index (0-based)
    pub section: usize,
    /// Block index within the section (0-based)
    pub block: usize,
    /// For paragraphs: the inline element index
    pub inline: usize,
    /// Character offset within the text run (byte offset)
    pub offset: usize,
}

impl CursorPosition {
    pub fn start() -> Self {
        Self {
            section: 0,
            block: 0,
            inline: 0,
            offset: 0,
        }
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self::start()
    }
}

/// The document cursor.
///
/// Tracks the current editing position and provides movement operations.
/// The cursor also remembers a "preferred column" for vertical movement
/// so that moving up/down through lines of varying length stays in
/// roughly the same horizontal position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Current position
    pub position: CursorPosition,
    /// Preferred horizontal offset (in twips) for vertical movement
    pub preferred_x: Option<i32>,
    /// Whether the cursor is visible (for blink state)
    pub visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: CursorPosition::start(),
            preferred_x: None,
            visible: true,
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction of cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Granularity of cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveUnit {
    /// Single grapheme cluster
    Character,
    /// Word boundary
    Word,
    /// Start/end of line (visual line)
    Line,
    /// Start/end of paragraph
    Paragraph,
    /// Start/end of page
    Page,
    /// Start/end of document
    Document,
}
