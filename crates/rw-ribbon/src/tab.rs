//! Ribbon tab definitions — defines the content of each built-in tab.

use crate::*;

/// Build the Home tab definition.
pub fn home_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Home,
        label: "Home".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Clipboard".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "paste".to_string(),
                        label: "Paste".to_string(),
                        icon: "edit-paste-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "paste_normal".to_string(), label: "Paste".to_string(), icon: None, description: Some("Paste content with formatting".to_string()) },
                            DropdownItem { id: "paste_plain".to_string(), label: "Paste Plain Text".to_string(), icon: None, description: Some("Paste as unformatted text".to_string()) },
                            DropdownItem { id: "paste_special".to_string(), label: "Paste Special...".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::Stack(vec![
                        RibbonItem::SmallButton { id: "cut".to_string(), label: "Cut".to_string(), icon: "edit-cut-symbolic".to_string(), tooltip: "Cut (Ctrl+X)".to_string() },
                        RibbonItem::SmallButton { id: "copy".to_string(), label: "Copy".to_string(), icon: "edit-copy-symbolic".to_string(), tooltip: "Copy (Ctrl+C)".to_string() },
                        RibbonItem::SmallButton { id: "format_painter".to_string(), label: "Format Painter".to_string(), icon: "format-painter-symbolic".to_string(), tooltip: "Format Painter".to_string() },
                    ]),
                ],
            },
            RibbonGroup {
                label: "Font".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::Stack(vec![
                        RibbonItem::Dropdown {
                            id: "font_family".to_string(),
                            label: "Font".to_string(),
                            items: Vec::new(), // populated dynamically from system fonts
                            selected: None,
                            width: 180,
                        },
                        RibbonItem::TextInput {
                            id: "font_size".to_string(),
                            label: "Size".to_string(),
                            value: "12".to_string(),
                            width: 48,
                        },
                    ]),
                    RibbonItem::Stack(vec![
                        RibbonItem::ToggleButton { id: "bold".to_string(), label: "Bold".to_string(), icon: "format-text-bold-symbolic".to_string(), tooltip: "Bold (Ctrl+B)".to_string(), pressed: false },
                        RibbonItem::ToggleButton { id: "italic".to_string(), label: "Italic".to_string(), icon: "format-text-italic-symbolic".to_string(), tooltip: "Italic (Ctrl+I)".to_string(), pressed: false },
                        RibbonItem::ToggleButton { id: "underline".to_string(), label: "Underline".to_string(), icon: "format-text-underline-symbolic".to_string(), tooltip: "Underline (Ctrl+U)".to_string(), pressed: false },
                        RibbonItem::ToggleButton { id: "strikethrough".to_string(), label: "Strikethrough".to_string(), icon: "format-text-strikethrough-symbolic".to_string(), tooltip: "Strikethrough".to_string(), pressed: false },
                    ]),
                    RibbonItem::Stack(vec![
                        RibbonItem::ColorPicker { id: "font_color".to_string(), label: "Font Color".to_string(), current_color: "#000000".to_string() },
                        RibbonItem::ColorPicker { id: "highlight_color".to_string(), label: "Highlight".to_string(), current_color: "#FFFF00".to_string() },
                    ]),
                ],
            },
            RibbonGroup {
                label: "Paragraph".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::Stack(vec![
                        RibbonItem::ToggleButton { id: "align_left".to_string(), label: "Align Left".to_string(), icon: "format-justify-left-symbolic".to_string(), tooltip: "Align Left (Ctrl+L)".to_string(), pressed: true },
                        RibbonItem::ToggleButton { id: "align_center".to_string(), label: "Center".to_string(), icon: "format-justify-center-symbolic".to_string(), tooltip: "Center (Ctrl+E)".to_string(), pressed: false },
                        RibbonItem::ToggleButton { id: "align_right".to_string(), label: "Align Right".to_string(), icon: "format-justify-right-symbolic".to_string(), tooltip: "Align Right (Ctrl+R)".to_string(), pressed: false },
                        RibbonItem::ToggleButton { id: "justify".to_string(), label: "Justify".to_string(), icon: "format-justify-fill-symbolic".to_string(), tooltip: "Justify (Ctrl+J)".to_string(), pressed: false },
                    ]),
                    RibbonItem::Stack(vec![
                        RibbonItem::SplitButton { id: "bullets".to_string(), label: "Bullets".to_string(), icon: "view-list-bullet-symbolic".to_string(), items: Vec::new() },
                        RibbonItem::SplitButton { id: "numbering".to_string(), label: "Numbering".to_string(), icon: "view-list-ordered-symbolic".to_string(), items: Vec::new() },
                        RibbonItem::SmallButton { id: "indent_decrease".to_string(), label: "Decrease Indent".to_string(), icon: "format-indent-less-symbolic".to_string(), tooltip: "Decrease Indent".to_string() },
                        RibbonItem::SmallButton { id: "indent_increase".to_string(), label: "Increase Indent".to_string(), icon: "format-indent-more-symbolic".to_string(), tooltip: "Increase Indent".to_string() },
                    ]),
                ],
            },
            RibbonGroup {
                label: "Styles".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::Gallery {
                        id: "quick_styles".to_string(),
                        label: "Styles".to_string(),
                        items: Vec::new(), // populated dynamically from style catalog
                        columns: 4,
                    },
                ],
            },
            RibbonGroup {
                label: "Editing".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "find".to_string(), label: "Find".to_string(), icon: "edit-find-symbolic".to_string(), tooltip: "Find (Ctrl+F)".to_string() },
                    RibbonItem::LargeButton { id: "replace".to_string(), label: "Replace".to_string(), icon: "edit-find-replace-symbolic".to_string(), tooltip: "Replace (Ctrl+H)".to_string() },
                ],
            },
        ],
    }
}

/// Build the Insert tab definition.
pub fn insert_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Insert,
        label: "Insert".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Pages".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "cover_page".to_string(), label: "Cover Page".to_string(), icon: "document-page-setup-symbolic".to_string(), tooltip: "Insert a cover page".to_string() },
                    RibbonItem::SmallButton { id: "page_break".to_string(), label: "Page Break".to_string(), icon: "insert-page-break-symbolic".to_string(), tooltip: "Insert Page Break (Ctrl+Enter)".to_string() },
                ],
            },
            RibbonGroup {
                label: "Tables".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_table".to_string(), label: "Table".to_string(), icon: "insert-table-symbolic".to_string(), tooltip: "Insert Table".to_string() },
                ],
            },
            RibbonGroup {
                label: "Illustrations".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_image".to_string(), label: "Image".to_string(), icon: "insert-image-symbolic".to_string(), tooltip: "Insert Image".to_string() },
                    RibbonItem::LargeButton { id: "insert_shape".to_string(), label: "Shapes".to_string(), icon: "insert-object-symbolic".to_string(), tooltip: "Insert Shape".to_string() },
                    RibbonItem::LargeButton { id: "insert_chart".to_string(), label: "Chart".to_string(), icon: "insert-chart-symbolic".to_string(), tooltip: "Insert Chart".to_string() },
                ],
            },
            RibbonGroup {
                label: "Links".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_hyperlink".to_string(), label: "Hyperlink".to_string(), icon: "insert-link-symbolic".to_string(), tooltip: "Insert Hyperlink (Ctrl+K)".to_string() },
                    RibbonItem::SmallButton { id: "insert_bookmark".to_string(), label: "Bookmark".to_string(), icon: "bookmark-new-symbolic".to_string(), tooltip: "Insert Bookmark".to_string() },
                    RibbonItem::SmallButton { id: "insert_cross_ref".to_string(), label: "Cross-reference".to_string(), icon: "insert-cross-ref-symbolic".to_string(), tooltip: "Insert Cross-reference".to_string() },
                ],
            },
            RibbonGroup {
                label: "Header & Footer".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton { id: "header".to_string(), label: "Header".to_string(), icon: "header-symbolic".to_string(), items: Vec::new() },
                    RibbonItem::SplitButton { id: "footer".to_string(), label: "Footer".to_string(), icon: "footer-symbolic".to_string(), items: Vec::new() },
                    RibbonItem::SplitButton { id: "page_number".to_string(), label: "Page Number".to_string(), icon: "page-number-symbolic".to_string(), items: Vec::new() },
                ],
            },
            RibbonGroup {
                label: "Text".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_textbox".to_string(), label: "Text Box".to_string(), icon: "insert-text-symbolic".to_string(), tooltip: "Insert Text Box".to_string() },
                    RibbonItem::SmallButton { id: "insert_date_time".to_string(), label: "Date & Time".to_string(), icon: "insert-date-symbolic".to_string(), tooltip: "Insert Date & Time".to_string() },
                    RibbonItem::SmallButton { id: "insert_special_char".to_string(), label: "Special Character".to_string(), icon: "insert-special-char-symbolic".to_string(), tooltip: "Insert Special Character".to_string() },
                ],
            },
        ],
    }
}
