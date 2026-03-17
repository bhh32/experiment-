//! Keyboard shortcut mapping.

use crate::messages::{
    CursorDirection, CursorMoveMsg, CursorUnit, KeyCode, KeyModifiers, Message,
};
use rw_editor::operations::DeleteDirection;

/// Map a key press + modifiers into an application Message.
/// Returns `None` if the key should be ignored.
pub fn map_key_to_message(key: KeyCode, mods: KeyModifiers) -> Option<Message> {
    let ctrl = mods.ctrl;
    let shift = mods.shift;
    let alt = mods.alt;

    match key {
        // --- Ctrl+letter shortcuts ---
        KeyCode::Character(ch) if ctrl => match ch.to_ascii_lowercase() {
            'n' => Some(Message::NewDocument),
            'o' => Some(Message::RequestOpenFile),
            's' => {
                if shift {
                    Some(Message::RequestSaveAs)
                } else {
                    Some(Message::Save)
                }
            }
            'z' => Some(Message::Undo),
            'y' => Some(Message::Redo),
            'x' => Some(Message::Cut),
            'c' => Some(Message::Copy),
            'v' => Some(Message::Paste(String::new())),
            'a' => Some(Message::SelectAll),
            'b' => Some(Message::RibbonAction("bold".into())),
            'i' => Some(Message::RibbonAction("italic".into())),
            'u' => Some(Message::RibbonAction("underline".into())),
            'l' => Some(Message::RibbonAction("align_left".into())),
            'e' => Some(Message::RibbonAction("align_center".into())),
            'r' => Some(Message::RibbonAction("align_right".into())),
            'j' => Some(Message::RibbonAction("justify".into())),
            'f' => Some(Message::OpenFind),
            'h' => Some(Message::OpenReplace),
            'p' => {
                if shift {
                    Some(Message::ToggleCommandPalette)
                } else {
                    Some(Message::Print)
                }
            }
            'w' => Some(Message::CloseDocument),
            _ => None,
        },

        // --- Printable characters ---
        KeyCode::Character(ch) if !ctrl && !alt => {
            Some(Message::InsertText(ch.to_string()))
        }

        // --- Enter ---
        KeyCode::Enter => {
            if ctrl {
                Some(Message::InsertPageBreak)
            } else if shift {
                Some(Message::InsertLineBreak)
            } else {
                Some(Message::InsertParagraphBreak)
            }
        }

        // --- Tab ---
        KeyCode::Tab => {
            if shift {
                Some(Message::DecreaseIndent)
            } else {
                Some(Message::InsertTab)
            }
        }

        // --- Backspace ---
        KeyCode::Backspace => {
            if ctrl {
                Some(Message::Delete(DeleteDirection::WordBackward))
            } else {
                Some(Message::Delete(DeleteDirection::Backward))
            }
        }

        // --- Delete ---
        KeyCode::Delete => {
            if ctrl {
                Some(Message::Delete(DeleteDirection::WordForward))
            } else {
                Some(Message::Delete(DeleteDirection::Forward))
            }
        }

        // --- Arrow keys ---
        KeyCode::Left => {
            let unit = if ctrl { CursorUnit::Word } else { CursorUnit::Character };
            Some(Message::CursorMove(CursorMoveMsg {
                direction: CursorDirection::Left,
                unit,
                extend_selection: shift,
            }))
        }
        KeyCode::Right => {
            let unit = if ctrl { CursorUnit::Word } else { CursorUnit::Character };
            Some(Message::CursorMove(CursorMoveMsg {
                direction: CursorDirection::Right,
                unit,
                extend_selection: shift,
            }))
        }
        KeyCode::Up => Some(Message::CursorMove(CursorMoveMsg {
            direction: CursorDirection::Up,
            unit: CursorUnit::Character,
            extend_selection: shift,
        })),
        KeyCode::Down => Some(Message::CursorMove(CursorMoveMsg {
            direction: CursorDirection::Down,
            unit: CursorUnit::Character,
            extend_selection: shift,
        })),

        // --- Home / End ---
        KeyCode::Home => {
            let unit = if ctrl { CursorUnit::Document } else { CursorUnit::Line };
            Some(Message::CursorMove(CursorMoveMsg {
                direction: CursorDirection::Left,
                unit,
                extend_selection: shift,
            }))
        }
        KeyCode::End => {
            let unit = if ctrl { CursorUnit::Document } else { CursorUnit::Line };
            Some(Message::CursorMove(CursorMoveMsg {
                direction: CursorDirection::Right,
                unit,
                extend_selection: shift,
            }))
        }

        // --- Page Up / Down ---
        KeyCode::PageUp => Some(Message::CursorMove(CursorMoveMsg {
            direction: CursorDirection::Up,
            unit: CursorUnit::Page,
            extend_selection: shift,
        })),
        KeyCode::PageDown => Some(Message::CursorMove(CursorMoveMsg {
            direction: CursorDirection::Down,
            unit: CursorUnit::Page,
            extend_selection: shift,
        })),

        // --- Insert ---
        KeyCode::Insert => Some(Message::None),

        // --- Escape ---
        KeyCode::Escape => Some(Message::CloseBackstage),

        // --- Function keys ---
        KeyCode::F1 => Some(Message::RibbonAction("help".into())),
        KeyCode::F5 if ctrl => Some(Message::ToggleNavigationPane),
        KeyCode::F7 => Some(Message::ToggleSpellCheck),

        _ => None,
    }
}
