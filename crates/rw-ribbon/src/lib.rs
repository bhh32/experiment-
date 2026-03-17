//! # rw-ribbon
//!
//! Custom Ribbon widget for Rust Writer, built on libcosmic/iced.
//!
//! Implements a Microsoft Word-style tabbed ribbon interface with:
//! - Fixed tabs: Home, Insert, Design, Layout, References, Mailings, Review, View
//! - Contextual tabs: Table Design, Table Layout, Picture Format, etc.
//! - Quick Access Toolbar (above the ribbon)
//! - Ribbon groups with labeled sections
//! - Ribbon collapse/expand (Ctrl+F1)
//! - Backstage view (File tab -> full-page overlay)
//! - Mini toolbar on text selection
//! - Three display modes: Full, Tabs Only, Auto-Hide
//!
//! ## Architecture
//!
//! The ribbon is a custom widget that composes libcosmic primitives
//! (buttons, dropdowns, segmented controls) into the ribbon layout.
//! It emits `RibbonMessage` variants that the application maps to
//! editor operations.

pub mod backstage;
pub mod group;
pub mod mini_toolbar;
pub mod tab;

/// A ribbon tab definition.
#[derive(Debug, Clone)]
pub struct RibbonTab {
    /// Tab identifier
    pub id: TabId,
    /// Display label
    pub label: String,
    /// Groups within this tab
    pub groups: Vec<RibbonGroup>,
    /// Whether this is a contextual tab (shown only in certain contexts)
    pub contextual: bool,
    /// Contextual tab color (for the accent strip)
    pub accent_color: Option<String>,
}

/// A group within a ribbon tab.
#[derive(Debug, Clone)]
pub struct RibbonGroup {
    /// Group label (displayed below the group)
    pub label: String,
    /// Items in this group
    pub items: Vec<RibbonItem>,
    /// Whether the group has a dialog launcher button
    pub has_dialog_launcher: bool,
}

/// An item in a ribbon group.
#[derive(Debug, Clone)]
pub enum RibbonItem {
    /// A large button with icon and text
    LargeButton {
        id: String,
        label: String,
        icon: String,
        tooltip: String,
    },
    /// A small button with icon only
    SmallButton {
        id: String,
        label: String,
        icon: String,
        tooltip: String,
    },
    /// A toggle button (pressed/unpressed state)
    ToggleButton {
        id: String,
        label: String,
        icon: String,
        tooltip: String,
        pressed: bool,
    },
    /// A split button (click for action, dropdown for more options)
    SplitButton {
        id: String,
        label: String,
        icon: String,
        items: Vec<DropdownItem>,
    },
    /// A dropdown selector
    Dropdown {
        id: String,
        label: String,
        items: Vec<DropdownItem>,
        selected: Option<usize>,
        width: u32,
    },
    /// A text input (e.g., font size)
    TextInput {
        id: String,
        label: String,
        value: String,
        width: u32,
    },
    /// A color picker button
    ColorPicker {
        id: String,
        label: String,
        current_color: String,
    },
    /// A gallery (grid of visual options, e.g., styles gallery)
    Gallery {
        id: String,
        label: String,
        items: Vec<GalleryItem>,
        columns: u32,
    },
    /// A separator line between items
    Separator,
    /// A vertical stack of small items
    Stack(Vec<RibbonItem>),
}

#[derive(Debug, Clone)]
pub struct DropdownItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GalleryItem {
    pub id: String,
    pub label: String,
    pub preview: Option<String>,
}

/// Predefined tab identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    File,
    Home,
    Insert,
    Design,
    Layout,
    References,
    Mailings,
    Review,
    View,
    // Contextual tabs
    TableDesign,
    TableLayout,
    PictureFormat,
    ShapeFormat,
    HeaderFooter,
    DrawingTools,
}

/// Ribbon display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonDisplayMode {
    /// Full ribbon always visible
    Full,
    /// Only tab labels visible; clicking a tab expands temporarily
    TabsOnly,
    /// Ribbon hidden; appears on tab click, disappears on document click
    AutoHide,
}
