//! Application message types.
//!
//! All user interactions and events are represented as messages
//! following the Elm Architecture (TEA / MVU) pattern.

use rw_ribbon::TabId;

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// No-op message
    None,

    // --- File operations ---
    /// Create a new empty document
    NewDocument,

    // --- Ribbon ---
    /// Switch the active ribbon tab
    TabChanged(TabId),

    // --- View ---
    /// Change zoom level
    ZoomChanged(u32),
    /// Toggle focus/zen mode
    ToggleFocusMode,
    /// Toggle command palette
    ToggleCommandPalette,
}
