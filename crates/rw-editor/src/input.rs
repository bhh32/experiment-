//! Input handling — mapping keyboard and mouse events to edit operations.

use crate::cursor::{MoveDirection, MoveUnit};
use crate::operations::{DeleteDirection, EditOperation};

/// A keyboard event from the UI layer.
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

/// Modifier keys state.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Super/Windows/Command key
}

/// Key codes for keyboard input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A character key
    Character(char),
    /// Function keys
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
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

/// A high-level editor action produced by mapping a key event.
///
/// This combines edit operations, cursor movements, and clipboard actions
/// into a single enum for dispatch by the `EditorState`.
#[derive(Debug, Clone)]
pub enum EditorAction {
    /// Perform an edit operation (insert, delete, format, etc.)
    Edit(EditOperation),
    /// Move the cursor
    MoveCursor {
        direction: MoveDirection,
        unit: MoveUnit,
        extend_selection: bool,
    },
    /// Select the entire document
    SelectAll,
    /// Cut selected content to clipboard
    Cut,
    /// Copy selected content to clipboard
    Copy,
    /// Paste from clipboard
    Paste,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Toggle overwrite / insert mode
    ToggleOverwrite,
    /// No action (e.g. Escape with no selection)
    Noop,
}

/// Map a `KeyEvent` to an `EditorAction`.
///
/// Returns `None` if the key event should be ignored (e.g. lone modifier keys).
pub fn map_key_to_operation(event: &KeyEvent) -> Option<EditorAction> {
    let shift = event.modifiers.shift;
    let ctrl = event.modifiers.ctrl;

    match &event.key {
        // ---- printable characters ----
        Key::Character(ch) => {
            if ctrl {
                // Ctrl+letter shortcuts
                match ch {
                    'a' | 'A' => Some(EditorAction::SelectAll),
                    'c' | 'C' => Some(EditorAction::Copy),
                    'x' | 'X' => Some(EditorAction::Cut),
                    'v' | 'V' => Some(EditorAction::Paste),
                    'z' | 'Z' => Some(EditorAction::Undo),
                    'y' | 'Y' => Some(EditorAction::Redo),
                    'b' | 'B' => Some(EditorAction::Edit(EditOperation::ApplyCharacterFormat(
                        crate::operations::CharacterFormatOp::ToggleBold,
                    ))),
                    'i' | 'I' => Some(EditorAction::Edit(EditOperation::ApplyCharacterFormat(
                        crate::operations::CharacterFormatOp::ToggleItalic,
                    ))),
                    'u' | 'U' => Some(EditorAction::Edit(EditOperation::ApplyCharacterFormat(
                        crate::operations::CharacterFormatOp::ToggleUnderline,
                    ))),
                    _ => None,
                }
            } else {
                Some(EditorAction::Edit(EditOperation::InsertText(ch.to_string())))
            }
        }

        // ---- Enter ----
        Key::Enter => {
            if shift {
                // Shift+Enter → line break
                Some(EditorAction::Edit(EditOperation::InsertLineBreak))
            } else if ctrl {
                // Ctrl+Enter → page break
                Some(EditorAction::Edit(EditOperation::InsertPageBreak))
            } else {
                Some(EditorAction::Edit(EditOperation::InsertParagraphBreak))
            }
        }

        // ---- Tab ----
        Key::Tab => Some(EditorAction::Edit(EditOperation::InsertTab)),

        // ---- Backspace ----
        Key::Backspace => {
            if ctrl {
                Some(EditorAction::Edit(EditOperation::Delete(
                    DeleteDirection::WordBackward,
                )))
            } else {
                Some(EditorAction::Edit(EditOperation::Delete(
                    DeleteDirection::Backward,
                )))
            }
        }

        // ---- Delete ----
        Key::Delete => {
            if ctrl {
                Some(EditorAction::Edit(EditOperation::Delete(
                    DeleteDirection::WordForward,
                )))
            } else {
                Some(EditorAction::Edit(EditOperation::Delete(
                    DeleteDirection::Forward,
                )))
            }
        }

        // ---- Arrow keys ----
        Key::Left => {
            let unit = if ctrl { MoveUnit::Word } else { MoveUnit::Character };
            Some(EditorAction::MoveCursor {
                direction: MoveDirection::Left,
                unit,
                extend_selection: shift,
            })
        }
        Key::Right => {
            let unit = if ctrl { MoveUnit::Word } else { MoveUnit::Character };
            Some(EditorAction::MoveCursor {
                direction: MoveDirection::Right,
                unit,
                extend_selection: shift,
            })
        }
        Key::Up => Some(EditorAction::MoveCursor {
            direction: MoveDirection::Up,
            unit: MoveUnit::Character,
            extend_selection: shift,
        }),
        Key::Down => Some(EditorAction::MoveCursor {
            direction: MoveDirection::Down,
            unit: MoveUnit::Character,
            extend_selection: shift,
        }),

        // ---- Home / End ----
        Key::Home => {
            let unit = if ctrl { MoveUnit::Document } else { MoveUnit::Line };
            Some(EditorAction::MoveCursor {
                direction: MoveDirection::Left,
                unit,
                extend_selection: shift,
            })
        }
        Key::End => {
            let unit = if ctrl { MoveUnit::Document } else { MoveUnit::Line };
            Some(EditorAction::MoveCursor {
                direction: MoveDirection::Right,
                unit,
                extend_selection: shift,
            })
        }

        // ---- Page Up / Down ----
        Key::PageUp => Some(EditorAction::MoveCursor {
            direction: MoveDirection::Up,
            unit: MoveUnit::Page,
            extend_selection: shift,
        }),
        Key::PageDown => Some(EditorAction::MoveCursor {
            direction: MoveDirection::Down,
            unit: MoveUnit::Page,
            extend_selection: shift,
        }),

        // ---- Insert ----
        Key::Insert => Some(EditorAction::ToggleOverwrite),

        // ---- Escape — clear selection, no edit ----
        Key::Escape => Some(EditorAction::Noop),

        // ---- Function keys (no default bindings) ----
        Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6
        | Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12 => None,
    }
}
