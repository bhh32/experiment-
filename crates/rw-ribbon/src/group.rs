//! Ribbon group layout logic and builder.
//!
//! Groups adapt their layout based on available space:
//! 1. Full: All items at natural size with labels
//! 2. Medium: Large buttons become small, text hidden
//! 3. Compact: Group collapses to a single dropdown button

use crate::{DropdownItem, RibbonGroup, RibbonItem};

/// Fluent builder for `RibbonGroup`.
///
/// # Example
///
/// ```rust
/// use rw_ribbon::group::GroupBuilder;
///
/// let group = GroupBuilder::new("Clipboard")
///     .button("paste", "Paste", "edit-paste-symbolic")
///     .button("cut", "Cut", "edit-cut-symbolic")
///     .separator()
///     .toggle("bold", "Bold", "format-text-bold-symbolic", false)
///     .build();
/// ```
pub struct GroupBuilder {
    label: String,
    items: Vec<RibbonItem>,
    has_dialog_launcher: bool,
}

impl GroupBuilder {
    /// Start building a group with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            has_dialog_launcher: false,
        }
    }

    /// Add a large button.
    pub fn button(mut self, id: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        let id = id.into();
        let label_str = label.into();
        let tooltip = format!("{}", label_str);
        self.items.push(RibbonItem::LargeButton {
            id,
            label: label_str,
            icon: icon.into(),
            tooltip,
        });
        self
    }

    /// Add a small button.
    pub fn small_button(mut self, id: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        let id = id.into();
        let label_str = label.into();
        let tooltip = format!("{}", label_str);
        self.items.push(RibbonItem::SmallButton {
            id,
            label: label_str,
            icon: icon.into(),
            tooltip,
        });
        self
    }

    /// Add a toggle button.
    pub fn toggle(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        pressed: bool,
    ) -> Self {
        let id = id.into();
        let label_str = label.into();
        let tooltip = format!("{}", label_str);
        self.items.push(RibbonItem::ToggleButton {
            id,
            label: label_str,
            icon: icon.into(),
            tooltip,
            pressed,
        });
        self
    }

    /// Add a split button (click = primary action, arrow = dropdown).
    pub fn split(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        items: Vec<DropdownItem>,
    ) -> Self {
        self.items.push(RibbonItem::SplitButton {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            items,
        });
        self
    }

    /// Add a dropdown selector.
    pub fn dropdown(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        items: Vec<DropdownItem>,
        width: u32,
    ) -> Self {
        self.items.push(RibbonItem::Dropdown {
            id: id.into(),
            label: label.into(),
            items,
            selected: None,
            width,
        });
        self
    }

    /// Add a text input field.
    pub fn text_input(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        width: u32,
    ) -> Self {
        self.items.push(RibbonItem::TextInput {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            width,
        });
        self
    }

    /// Add a color picker.
    pub fn color_picker(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        current_color: impl Into<String>,
    ) -> Self {
        self.items.push(RibbonItem::ColorPicker {
            id: id.into(),
            label: label.into(),
            current_color: current_color.into(),
        });
        self
    }

    /// Add a vertical separator line.
    pub fn separator(mut self) -> Self {
        self.items.push(RibbonItem::Separator);
        self
    }

    /// Push a raw `RibbonItem` directly.
    pub fn item(mut self, item: RibbonItem) -> Self {
        self.items.push(item);
        self
    }

    /// Enable the dialog launcher arrow.
    pub fn with_dialog_launcher(mut self) -> Self {
        self.has_dialog_launcher = true;
        self
    }

    /// Consume the builder and produce the `RibbonGroup`.
    pub fn build(self) -> RibbonGroup {
        RibbonGroup {
            label: self.label,
            items: self.items,
            has_dialog_launcher: self.has_dialog_launcher,
        }
    }
}

// ---------------------------------------------------------------------------
// Display-mode helpers
// ---------------------------------------------------------------------------

/// The amount of horizontal space a group needs at each display level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupDisplayMode {
    /// All items shown at full size with labels
    Full,
    /// Large buttons shrink to small, labels hidden
    Medium,
    /// Entire group collapses to a single dropdown button
    Compact,
}

/// Determine the appropriate display mode given the available pixels.
pub fn display_mode_for_width(available_px: f64, full_px: f64, medium_px: f64) -> GroupDisplayMode {
    if available_px >= full_px {
        GroupDisplayMode::Full
    } else if available_px >= medium_px {
        GroupDisplayMode::Medium
    } else {
        GroupDisplayMode::Compact
    }
}
