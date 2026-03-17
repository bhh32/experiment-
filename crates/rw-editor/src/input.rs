//! Input handling — mapping keyboard and mouse events to edit operations.

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
