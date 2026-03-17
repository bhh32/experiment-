//! Mini toolbar — the floating formatting toolbar that appears on text selection.
//!
//! Shows commonly used formatting options (font, size, bold, italic, color)
//! in a compact floating toolbar above the selection.

use crate::{DropdownItem, RibbonItem};

/// Items available in the mini toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniToolbarItem {
    FontFamily,
    FontSize,
    IncreaseFontSize,
    DecreaseFontSize,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Subscript,
    Superscript,
    FontColor,
    HighlightColor,
    BulletList,
    NumberedList,
    AlignLeft,
    AlignCenter,
    AlignRight,
    IndentDecrease,
    IndentIncrease,
    StylesGallery,
}

/// The current formatting state used to set toggle-button pressed states.
#[derive(Debug, Clone)]
pub struct FormattingState {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub font_family: String,
    pub font_size: f64,
    pub font_color: String,
    pub highlight_color: String,
    pub align_left: bool,
    pub align_center: bool,
    pub align_right: bool,
    pub justify: bool,
}

impl Default for FormattingState {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            superscript: false,
            subscript: false,
            font_family: "Calibri".to_string(),
            font_size: 12.0,
            font_color: "#000000".to_string(),
            highlight_color: "#FFFF00".to_string(),
            align_left: true,
            align_center: false,
            align_right: false,
            justify: false,
        }
    }
}

/// The mini toolbar widget state.
#[derive(Debug, Clone)]
pub struct MiniToolbar {
    /// Whether the toolbar is currently visible
    pub visible: bool,
    /// Position (x, y) in screen pixels
    pub position: (f64, f64),
    /// Current formatting state, used to initialise toggle buttons
    pub formatting: FormattingState,
    /// Opacity (0.0 = invisible, 1.0 = fully visible)
    /// The toolbar fades in when the selection stabilises.
    pub opacity: f64,
}

impl MiniToolbar {
    /// Create a new, hidden mini toolbar.
    pub fn new() -> Self {
        Self {
            visible: false,
            position: (0.0, 0.0),
            formatting: FormattingState::default(),
            opacity: 0.0,
        }
    }

    /// Show the toolbar at the given screen position with the given formatting state.
    pub fn show(&mut self, x: f64, y: f64, formatting: FormattingState) {
        self.visible = true;
        self.position = (x, y);
        self.formatting = formatting;
        self.opacity = 1.0;
    }

    /// Hide the toolbar.
    pub fn hide(&mut self) {
        self.visible = false;
        self.opacity = 0.0;
    }

    /// Return the `RibbonItem`s to display for the given formatting state.
    ///
    /// The order matches the standard mini-toolbar layout found in
    /// Microsoft Word / LibreOffice Writer.
    pub fn get_items(formatting_state: &FormattingState) -> Vec<RibbonItem> {
        vec![
            // Font family
            RibbonItem::Dropdown {
                id: "mt_font_family".to_string(),
                label: "Font".to_string(),
                items: Vec::new(),
                selected: None,
                width: 120,
            },
            // Font size
            RibbonItem::TextInput {
                id: "mt_font_size".to_string(),
                label: "Size".to_string(),
                value: format!("{}", formatting_state.font_size as u32),
                width: 40,
            },
            // Increase / decrease font size
            RibbonItem::Stack(vec![
                RibbonItem::SmallButton {
                    id: "mt_font_grow".to_string(),
                    label: "Grow".to_string(),
                    icon: "format-text-larger-symbolic".to_string(),
                    tooltip: "Increase Font Size (Ctrl+>)".to_string(),
                },
                RibbonItem::SmallButton {
                    id: "mt_font_shrink".to_string(),
                    label: "Shrink".to_string(),
                    icon: "format-text-smaller-symbolic".to_string(),
                    tooltip: "Decrease Font Size (Ctrl+<)".to_string(),
                },
            ]),
            RibbonItem::Separator,
            // Bold / Italic / Underline
            RibbonItem::ToggleButton {
                id: "mt_bold".to_string(),
                label: "Bold".to_string(),
                icon: "format-text-bold-symbolic".to_string(),
                tooltip: "Bold (Ctrl+B)".to_string(),
                pressed: formatting_state.bold,
            },
            RibbonItem::ToggleButton {
                id: "mt_italic".to_string(),
                label: "Italic".to_string(),
                icon: "format-text-italic-symbolic".to_string(),
                tooltip: "Italic (Ctrl+I)".to_string(),
                pressed: formatting_state.italic,
            },
            RibbonItem::ToggleButton {
                id: "mt_underline".to_string(),
                label: "Underline".to_string(),
                icon: "format-text-underline-symbolic".to_string(),
                tooltip: "Underline (Ctrl+U)".to_string(),
                pressed: formatting_state.underline,
            },
            RibbonItem::Separator,
            // Colors
            RibbonItem::ColorPicker {
                id: "mt_font_color".to_string(),
                label: "Font Color".to_string(),
                current_color: formatting_state.font_color.clone(),
            },
            RibbonItem::ColorPicker {
                id: "mt_highlight".to_string(),
                label: "Highlight".to_string(),
                current_color: formatting_state.highlight_color.clone(),
            },
            RibbonItem::Separator,
            // Lists
            RibbonItem::SmallButton {
                id: "mt_bullets".to_string(),
                label: "Bullets".to_string(),
                icon: "view-list-bullet-symbolic".to_string(),
                tooltip: "Bulleted List".to_string(),
            },
            RibbonItem::SmallButton {
                id: "mt_numbering".to_string(),
                label: "Numbering".to_string(),
                icon: "view-list-ordered-symbolic".to_string(),
                tooltip: "Numbered List".to_string(),
            },
            RibbonItem::Separator,
            // Indent
            RibbonItem::SmallButton {
                id: "mt_indent_decrease".to_string(),
                label: "Decrease Indent".to_string(),
                icon: "format-indent-less-symbolic".to_string(),
                tooltip: "Decrease Indent".to_string(),
            },
            RibbonItem::SmallButton {
                id: "mt_indent_increase".to_string(),
                label: "Increase Indent".to_string(),
                icon: "format-indent-more-symbolic".to_string(),
                tooltip: "Increase Indent".to_string(),
            },
            // Style picker
            RibbonItem::SplitButton {
                id: "mt_styles".to_string(),
                label: "Styles".to_string(),
                icon: "font-select-symbolic".to_string(),
                items: vec![
                    DropdownItem { id: "style_normal".to_string(), label: "Normal".to_string(), icon: None, description: None },
                    DropdownItem { id: "style_heading1".to_string(), label: "Heading 1".to_string(), icon: None, description: None },
                    DropdownItem { id: "style_heading2".to_string(), label: "Heading 2".to_string(), icon: None, description: None },
                ],
            },
        ]
    }
}

impl Default for MiniToolbar {
    fn default() -> Self {
        Self::new()
    }
}
