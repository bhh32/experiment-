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
                            items: Vec::new(),
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
                        items: Vec::new(),
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

/// Build the Design tab definition.
pub fn design_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Design,
        label: "Design".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Themes".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::Gallery {
                        id: "themes_gallery".to_string(),
                        label: "Themes".to_string(),
                        items: vec![
                            GalleryItem { id: "theme_office".to_string(), label: "Office".to_string(), preview: None },
                            GalleryItem { id: "theme_grayscale".to_string(), label: "Grayscale".to_string(), preview: None },
                            GalleryItem { id: "theme_facet".to_string(), label: "Facet".to_string(), preview: None },
                            GalleryItem { id: "theme_integral".to_string(), label: "Integral".to_string(), preview: None },
                        ],
                        columns: 4,
                    },
                ],
            },
            RibbonGroup {
                label: "Document Formatting".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::Gallery {
                        id: "style_set_gallery".to_string(),
                        label: "Style Set".to_string(),
                        items: vec![
                            GalleryItem { id: "style_default".to_string(), label: "Default".to_string(), preview: None },
                            GalleryItem { id: "style_basic".to_string(), label: "Basic".to_string(), preview: None },
                            GalleryItem { id: "style_word2013".to_string(), label: "Word 2013".to_string(), preview: None },
                        ],
                        columns: 3,
                    },
                    RibbonItem::SplitButton {
                        id: "theme_colors".to_string(),
                        label: "Colors".to_string(),
                        icon: "color-scheme-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "colors_office".to_string(), label: "Office".to_string(), icon: None, description: None },
                            DropdownItem { id: "colors_blue".to_string(), label: "Blue Warm".to_string(), icon: None, description: None },
                            DropdownItem { id: "colors_custom".to_string(), label: "Customize Colors…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "theme_fonts".to_string(),
                        label: "Fonts".to_string(),
                        icon: "font-select-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "fonts_calibri".to_string(), label: "Calibri / Calibri Light".to_string(), icon: None, description: None },
                            DropdownItem { id: "fonts_times".to_string(), label: "Times New Roman".to_string(), icon: None, description: None },
                            DropdownItem { id: "fonts_custom".to_string(), label: "Customize Fonts…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "paragraph_spacing".to_string(),
                        label: "Paragraph Spacing".to_string(),
                        icon: "format-indent-more-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "spacing_default".to_string(), label: "Default".to_string(), icon: None, description: None },
                            DropdownItem { id: "spacing_compact".to_string(), label: "Compact".to_string(), icon: None, description: None },
                            DropdownItem { id: "spacing_relaxed".to_string(), label: "Relaxed".to_string(), icon: None, description: None },
                        ],
                    },
                ],
            },
            RibbonGroup {
                label: "Page Background".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "watermark".to_string(), label: "Watermark".to_string(), icon: "watermark-symbolic".to_string(), tooltip: "Add a watermark behind page content".to_string() },
                    RibbonItem::ColorPicker { id: "page_color".to_string(), label: "Page Color".to_string(), current_color: "#FFFFFF".to_string() },
                    RibbonItem::LargeButton { id: "page_borders".to_string(), label: "Page Borders".to_string(), icon: "draw-border-symbolic".to_string(), tooltip: "Add borders around pages".to_string() },
                ],
            },
        ],
    }
}

/// Build the Layout tab definition.
pub fn layout_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Layout,
        label: "Layout".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Page Setup".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "margins".to_string(),
                        label: "Margins".to_string(),
                        icon: "page-margins-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "margins_normal".to_string(), label: "Normal (1\" all)".to_string(), icon: None, description: None },
                            DropdownItem { id: "margins_narrow".to_string(), label: "Narrow (0.5\" all)".to_string(), icon: None, description: None },
                            DropdownItem { id: "margins_wide".to_string(), label: "Wide (2\" L/R)".to_string(), icon: None, description: None },
                            DropdownItem { id: "margins_custom".to_string(), label: "Custom Margins…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "orientation".to_string(),
                        label: "Orientation".to_string(),
                        icon: "page-orientation-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "orientation_portrait".to_string(), label: "Portrait".to_string(), icon: None, description: None },
                            DropdownItem { id: "orientation_landscape".to_string(), label: "Landscape".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "page_size".to_string(),
                        label: "Size".to_string(),
                        icon: "document-page-setup-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "size_letter".to_string(), label: "Letter (8.5\" × 11\")".to_string(), icon: None, description: None },
                            DropdownItem { id: "size_a4".to_string(), label: "A4 (210mm × 297mm)".to_string(), icon: None, description: None },
                            DropdownItem { id: "size_legal".to_string(), label: "Legal (8.5\" × 14\")".to_string(), icon: None, description: None },
                            DropdownItem { id: "size_custom".to_string(), label: "More Paper Sizes…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "columns".to_string(),
                        label: "Columns".to_string(),
                        icon: "format-columns-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "columns_one".to_string(), label: "One".to_string(), icon: None, description: None },
                            DropdownItem { id: "columns_two".to_string(), label: "Two".to_string(), icon: None, description: None },
                            DropdownItem { id: "columns_three".to_string(), label: "Three".to_string(), icon: None, description: None },
                            DropdownItem { id: "columns_custom".to_string(), label: "More Columns…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "breaks".to_string(),
                        label: "Breaks".to_string(),
                        icon: "insert-page-break-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "break_page".to_string(), label: "Page".to_string(), icon: None, description: Some("Mark the point at which one page ends and the next page begins.".to_string()) },
                            DropdownItem { id: "break_column".to_string(), label: "Column".to_string(), icon: None, description: None },
                            DropdownItem { id: "break_section_next".to_string(), label: "Next Page Section".to_string(), icon: None, description: None },
                            DropdownItem { id: "break_section_cont".to_string(), label: "Continuous Section".to_string(), icon: None, description: None },
                        ],
                    },
                ],
            },
            RibbonGroup {
                label: "Paragraph".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::Stack(vec![
                        RibbonItem::TextInput { id: "indent_left".to_string(), label: "Indent Left".to_string(), value: "0\"".to_string(), width: 72 },
                        RibbonItem::TextInput { id: "indent_right".to_string(), label: "Indent Right".to_string(), value: "0\"".to_string(), width: 72 },
                    ]),
                    RibbonItem::Stack(vec![
                        RibbonItem::TextInput { id: "space_before".to_string(), label: "Before".to_string(), value: "0pt".to_string(), width: 64 },
                        RibbonItem::TextInput { id: "space_after".to_string(), label: "After".to_string(), value: "8pt".to_string(), width: 64 },
                    ]),
                ],
            },
            RibbonGroup {
                label: "Arrange".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "position".to_string(),
                        label: "Position".to_string(),
                        icon: "object-position-symbolic".to_string(),
                        items: Vec::new(),
                    },
                    RibbonItem::SplitButton {
                        id: "wrap_text".to_string(),
                        label: "Wrap Text".to_string(),
                        icon: "wrap-text-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "wrap_inline".to_string(), label: "In Line with Text".to_string(), icon: None, description: None },
                            DropdownItem { id: "wrap_square".to_string(), label: "Square".to_string(), icon: None, description: None },
                            DropdownItem { id: "wrap_tight".to_string(), label: "Tight".to_string(), icon: None, description: None },
                            DropdownItem { id: "wrap_through".to_string(), label: "Through".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SmallButton { id: "send_backward".to_string(), label: "Send Backward".to_string(), icon: "object-send-back-symbolic".to_string(), tooltip: "Send object one step back".to_string() },
                    RibbonItem::SmallButton { id: "bring_forward".to_string(), label: "Bring Forward".to_string(), icon: "object-bring-front-symbolic".to_string(), tooltip: "Bring object one step forward".to_string() },
                    RibbonItem::LargeButton { id: "align".to_string(), label: "Align".to_string(), icon: "object-align-symbolic".to_string(), tooltip: "Align objects".to_string() },
                    RibbonItem::LargeButton { id: "group".to_string(), label: "Group".to_string(), icon: "object-group-symbolic".to_string(), tooltip: "Group objects".to_string() },
                ],
            },
        ],
    }
}

/// Build the References tab definition.
pub fn references_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::References,
        label: "References".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Table of Contents".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "toc".to_string(),
                        label: "Table of Contents".to_string(),
                        icon: "view-list-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "toc_auto1".to_string(), label: "Automatic Table 1".to_string(), icon: None, description: None },
                            DropdownItem { id: "toc_auto2".to_string(), label: "Automatic Table 2".to_string(), icon: None, description: None },
                            DropdownItem { id: "toc_manual".to_string(), label: "Manual Table".to_string(), icon: None, description: None },
                            DropdownItem { id: "toc_custom".to_string(), label: "Custom Table of Contents…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SmallButton { id: "add_text".to_string(), label: "Add Text".to_string(), icon: "list-add-symbolic".to_string(), tooltip: "Add the current paragraph to the TOC".to_string() },
                    RibbonItem::SmallButton { id: "update_table".to_string(), label: "Update Table".to_string(), icon: "view-refresh-symbolic".to_string(), tooltip: "Update the table of contents".to_string() },
                ],
            },
            RibbonGroup {
                label: "Footnotes".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_footnote".to_string(), label: "Insert Footnote".to_string(), icon: "insert-footnote-symbolic".to_string(), tooltip: "Insert Footnote (Alt+Ctrl+F)".to_string() },
                    RibbonItem::LargeButton { id: "insert_endnote".to_string(), label: "Insert Endnote".to_string(), icon: "insert-endnote-symbolic".to_string(), tooltip: "Insert Endnote (Alt+Ctrl+D)".to_string() },
                    RibbonItem::SmallButton { id: "next_footnote".to_string(), label: "Next Footnote".to_string(), icon: "go-down-symbolic".to_string(), tooltip: "Go to next footnote".to_string() },
                    RibbonItem::SmallButton { id: "show_notes".to_string(), label: "Show Notes".to_string(), icon: "dialog-information-symbolic".to_string(), tooltip: "Show footnotes and endnotes".to_string() },
                ],
            },
            RibbonGroup {
                label: "Citations & Bibliography".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_citation".to_string(), label: "Insert Citation".to_string(), icon: "insert-citation-symbolic".to_string(), tooltip: "Cite a book, article, or other source".to_string() },
                    RibbonItem::LargeButton { id: "manage_sources".to_string(), label: "Manage Sources".to_string(), icon: "document-properties-symbolic".to_string(), tooltip: "View the list of all sources cited".to_string() },
                    RibbonItem::Dropdown {
                        id: "bibliography_style".to_string(),
                        label: "Style".to_string(),
                        items: vec![
                            DropdownItem { id: "bib_apa".to_string(), label: "APA".to_string(), icon: None, description: None },
                            DropdownItem { id: "bib_mla".to_string(), label: "MLA".to_string(), icon: None, description: None },
                            DropdownItem { id: "bib_chicago".to_string(), label: "Chicago".to_string(), icon: None, description: None },
                        ],
                        selected: Some(0),
                        width: 100,
                    },
                    RibbonItem::SplitButton {
                        id: "bibliography".to_string(),
                        label: "Bibliography".to_string(),
                        icon: "view-list-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "bib_built_in".to_string(), label: "Bibliography".to_string(), icon: None, description: None },
                            DropdownItem { id: "bib_works_cited".to_string(), label: "Works Cited".to_string(), icon: None, description: None },
                        ],
                    },
                ],
            },
            RibbonGroup {
                label: "Captions".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "insert_caption".to_string(), label: "Insert Caption".to_string(), icon: "insert-caption-symbolic".to_string(), tooltip: "Add a caption to a picture, table, or equation".to_string() },
                    RibbonItem::LargeButton { id: "insert_table_of_figures".to_string(), label: "Table of Figures".to_string(), icon: "view-list-symbolic".to_string(), tooltip: "Insert a table of figures".to_string() },
                    RibbonItem::SmallButton { id: "cross_reference".to_string(), label: "Cross-reference".to_string(), icon: "insert-link-symbolic".to_string(), tooltip: "Insert cross-reference".to_string() },
                ],
            },
            RibbonGroup {
                label: "Index".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "mark_entry".to_string(), label: "Mark Entry".to_string(), icon: "tag-new-symbolic".to_string(), tooltip: "Mark an index entry".to_string() },
                    RibbonItem::LargeButton { id: "insert_index".to_string(), label: "Insert Index".to_string(), icon: "view-list-symbolic".to_string(), tooltip: "Insert an index".to_string() },
                    RibbonItem::SmallButton { id: "update_index".to_string(), label: "Update Index".to_string(), icon: "view-refresh-symbolic".to_string(), tooltip: "Update the index".to_string() },
                ],
            },
        ],
    }
}

/// Build the Review tab definition.
pub fn review_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Review,
        label: "Review".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Proofing".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "spelling".to_string(), label: "Spelling & Grammar".to_string(), icon: "tools-check-spelling-symbolic".to_string(), tooltip: "Check spelling and grammar (F7)".to_string() },
                    RibbonItem::LargeButton { id: "thesaurus".to_string(), label: "Thesaurus".to_string(), icon: "applications-education-language-symbolic".to_string(), tooltip: "Suggest synonyms (Shift+F7)".to_string() },
                    RibbonItem::SmallButton { id: "word_count_btn".to_string(), label: "Word Count".to_string(), icon: "view-list-symbolic".to_string(), tooltip: "Word and character count".to_string() },
                ],
            },
            RibbonGroup {
                label: "Comments".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "new_comment".to_string(), label: "New Comment".to_string(), icon: "insert-comment-symbolic".to_string(), tooltip: "Insert a new comment (Ctrl+Alt+M)".to_string() },
                    RibbonItem::SmallButton { id: "delete_comment".to_string(), label: "Delete".to_string(), icon: "edit-delete-symbolic".to_string(), tooltip: "Delete comment".to_string() },
                    RibbonItem::SmallButton { id: "prev_comment".to_string(), label: "Previous".to_string(), icon: "go-previous-symbolic".to_string(), tooltip: "Go to previous comment".to_string() },
                    RibbonItem::SmallButton { id: "next_comment".to_string(), label: "Next".to_string(), icon: "go-next-symbolic".to_string(), tooltip: "Go to next comment".to_string() },
                    RibbonItem::ToggleButton { id: "show_comments".to_string(), label: "Show Comments".to_string(), icon: "view-list-symbolic".to_string(), tooltip: "Show or hide all comments".to_string(), pressed: true },
                ],
            },
            RibbonGroup {
                label: "Tracking".to_string(),
                has_dialog_launcher: true,
                items: vec![
                    RibbonItem::LargeButton { id: "track_changes".to_string(), label: "Track Changes".to_string(), icon: "document-edit-symbolic".to_string(), tooltip: "Track changes to the document (Ctrl+Shift+E)".to_string() },
                    RibbonItem::SplitButton {
                        id: "show_markup".to_string(),
                        label: "Show Markup".to_string(),
                        icon: "view-list-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "markup_comments".to_string(), label: "Comments".to_string(), icon: None, description: None },
                            DropdownItem { id: "markup_insertions".to_string(), label: "Insertions and Deletions".to_string(), icon: None, description: None },
                            DropdownItem { id: "markup_formatting".to_string(), label: "Formatting".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "display_for_review".to_string(),
                        label: "Display for Review".to_string(),
                        icon: "view-paged-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "dfr_final".to_string(), label: "Final: Show Markup".to_string(), icon: None, description: None },
                            DropdownItem { id: "dfr_final_clean".to_string(), label: "Final".to_string(), icon: None, description: None },
                            DropdownItem { id: "dfr_original".to_string(), label: "Original".to_string(), icon: None, description: None },
                        ],
                    },
                ],
            },
            RibbonGroup {
                label: "Changes".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "accept".to_string(), label: "Accept".to_string(), icon: "emblem-ok-symbolic".to_string(), tooltip: "Accept the tracked change".to_string() },
                    RibbonItem::LargeButton { id: "reject".to_string(), label: "Reject".to_string(), icon: "dialog-cancel-symbolic".to_string(), tooltip: "Reject the tracked change".to_string() },
                    RibbonItem::SmallButton { id: "prev_change".to_string(), label: "Previous".to_string(), icon: "go-previous-symbolic".to_string(), tooltip: "Go to previous change".to_string() },
                    RibbonItem::SmallButton { id: "next_change".to_string(), label: "Next".to_string(), icon: "go-next-symbolic".to_string(), tooltip: "Go to next change".to_string() },
                ],
            },
            RibbonGroup {
                label: "Compare".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "compare".to_string(),
                        label: "Compare".to_string(),
                        icon: "view-split-left-right-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "compare_documents".to_string(), label: "Compare…".to_string(), icon: None, description: Some("Compare two versions of a document".to_string()) },
                            DropdownItem { id: "combine_documents".to_string(), label: "Combine…".to_string(), icon: None, description: Some("Combine revisions from multiple authors".to_string()) },
                        ],
                    },
                ],
            },
            RibbonGroup {
                label: "Protect".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "protect_document".to_string(), label: "Protect Document".to_string(), icon: "document-encrypt-symbolic".to_string(), tooltip: "Restrict access to the document".to_string() },
                ],
            },
        ],
    }
}

/// Build the View tab definition.
pub fn view_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::View,
        label: "View".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Views".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::ToggleButton { id: "view_print_layout".to_string(), label: "Print Layout".to_string(), icon: "document-print-symbolic".to_string(), tooltip: "Print Layout view".to_string(), pressed: true },
                    RibbonItem::ToggleButton { id: "view_web_layout".to_string(), label: "Web Layout".to_string(), icon: "applications-internet-symbolic".to_string(), tooltip: "Web Layout view".to_string(), pressed: false },
                    RibbonItem::ToggleButton { id: "view_outline".to_string(), label: "Outline".to_string(), icon: "view-list-tree-symbolic".to_string(), tooltip: "Outline view".to_string(), pressed: false },
                    RibbonItem::ToggleButton { id: "view_draft".to_string(), label: "Draft".to_string(), icon: "document-edit-symbolic".to_string(), tooltip: "Draft view".to_string(), pressed: false },
                    RibbonItem::ToggleButton { id: "view_read_mode".to_string(), label: "Read Mode".to_string(), icon: "view-fullscreen-symbolic".to_string(), tooltip: "Read Mode view".to_string(), pressed: false },
                    RibbonItem::ToggleButton { id: "view_focus".to_string(), label: "Focus".to_string(), icon: "zoom-fit-best-symbolic".to_string(), tooltip: "Immersive Focus mode".to_string(), pressed: false },
                ],
            },
            RibbonGroup {
                label: "Show".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::ToggleButton { id: "show_ruler".to_string(), label: "Ruler".to_string(), icon: "ruler-symbolic".to_string(), tooltip: "Show the ruler".to_string(), pressed: true },
                    RibbonItem::ToggleButton { id: "show_gridlines".to_string(), label: "Gridlines".to_string(), icon: "view-grid-symbolic".to_string(), tooltip: "Show gridlines".to_string(), pressed: false },
                    RibbonItem::ToggleButton { id: "show_navigation".to_string(), label: "Navigation Pane".to_string(), icon: "sidebar-show-symbolic".to_string(), tooltip: "Toggle the Navigation pane".to_string(), pressed: false },
                ],
            },
            RibbonGroup {
                label: "Zoom".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "zoom_dialog".to_string(), label: "Zoom".to_string(), icon: "zoom-symbolic".to_string(), tooltip: "Change the zoom level".to_string() },
                    RibbonItem::SmallButton { id: "zoom_100".to_string(), label: "100%".to_string(), icon: "zoom-original-symbolic".to_string(), tooltip: "Zoom to 100%".to_string() },
                    RibbonItem::SmallButton { id: "zoom_page".to_string(), label: "One Page".to_string(), icon: "zoom-fit-best-symbolic".to_string(), tooltip: "Fit one page in the window".to_string() },
                    RibbonItem::SmallButton { id: "zoom_width".to_string(), label: "Page Width".to_string(), icon: "zoom-fit-page-symbolic".to_string(), tooltip: "Fit the page width".to_string() },
                ],
            },
            RibbonGroup {
                label: "Window".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SmallButton { id: "new_window".to_string(), label: "New Window".to_string(), icon: "window-new-symbolic".to_string(), tooltip: "Open a new window for this document".to_string() },
                    RibbonItem::SmallButton { id: "arrange_all".to_string(), label: "Arrange All".to_string(), icon: "view-grid-symbolic".to_string(), tooltip: "Tile all open windows side by side".to_string() },
                    RibbonItem::SmallButton { id: "split_window".to_string(), label: "Split".to_string(), icon: "view-split-top-bottom-symbolic".to_string(), tooltip: "Split the window into two scrolling panes".to_string() },
                    RibbonItem::SmallButton { id: "switch_windows".to_string(), label: "Switch Windows".to_string(), icon: "view-paged-symbolic".to_string(), tooltip: "Switch to a different open document".to_string() },
                ],
            },
        ],
    }
}

/// Build the Mailings tab definition.
pub fn mailings_tab() -> RibbonTab {
    RibbonTab {
        id: TabId::Mailings,
        label: "Mailings".to_string(),
        contextual: false,
        accent_color: None,
        groups: vec![
            RibbonGroup {
                label: "Create".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::LargeButton { id: "envelopes".to_string(), label: "Envelopes".to_string(), icon: "mail-send-symbolic".to_string(), tooltip: "Create and print envelopes".to_string() },
                    RibbonItem::LargeButton { id: "labels".to_string(), label: "Labels".to_string(), icon: "label-symbolic".to_string(), tooltip: "Create and print labels".to_string() },
                ],
            },
            RibbonGroup {
                label: "Start Mail Merge".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "start_mail_merge".to_string(),
                        label: "Start Mail Merge".to_string(),
                        icon: "mail-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "mm_letters".to_string(), label: "Letters".to_string(), icon: None, description: None },
                            DropdownItem { id: "mm_email".to_string(), label: "E-mail Messages".to_string(), icon: None, description: None },
                            DropdownItem { id: "mm_envelopes".to_string(), label: "Envelopes".to_string(), icon: None, description: None },
                            DropdownItem { id: "mm_labels".to_string(), label: "Labels".to_string(), icon: None, description: None },
                            DropdownItem { id: "mm_directory".to_string(), label: "Directory".to_string(), icon: None, description: None },
                            DropdownItem { id: "mm_wizard".to_string(), label: "Step-by-Step Mail Merge Wizard…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SplitButton {
                        id: "select_recipients".to_string(),
                        label: "Select Recipients".to_string(),
                        icon: "system-users-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "rec_new".to_string(), label: "Type a New List…".to_string(), icon: None, description: None },
                            DropdownItem { id: "rec_existing".to_string(), label: "Use an Existing List…".to_string(), icon: None, description: None },
                        ],
                    },
                    RibbonItem::SmallButton { id: "edit_recipient_list".to_string(), label: "Edit Recipient List".to_string(), icon: "document-edit-symbolic".to_string(), tooltip: "Edit the list of recipients".to_string() },
                ],
            },
            RibbonGroup {
                label: "Write & Insert Fields".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::ToggleButton { id: "highlight_merge_fields".to_string(), label: "Highlight Merge Fields".to_string(), icon: "format-text-highlight-symbolic".to_string(), tooltip: "Highlight merge fields".to_string(), pressed: false },
                    RibbonItem::LargeButton { id: "address_block".to_string(), label: "Address Block".to_string(), icon: "mail-symbolic".to_string(), tooltip: "Insert address block".to_string() },
                    RibbonItem::LargeButton { id: "greeting_line".to_string(), label: "Greeting Line".to_string(), icon: "format-text-symbolic".to_string(), tooltip: "Insert greeting line".to_string() },
                    RibbonItem::SplitButton {
                        id: "insert_merge_field".to_string(),
                        label: "Insert Merge Field".to_string(),
                        icon: "insert-field-symbolic".to_string(),
                        items: Vec::new(), // Populated from data source
                    },
                ],
            },
            RibbonGroup {
                label: "Preview Results".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::ToggleButton { id: "preview_results".to_string(), label: "Preview Results".to_string(), icon: "media-playback-start-symbolic".to_string(), tooltip: "Preview the merged result".to_string(), pressed: false },
                    RibbonItem::SmallButton { id: "find_recipient".to_string(), label: "Find Recipient".to_string(), icon: "edit-find-symbolic".to_string(), tooltip: "Find a specific recipient".to_string() },
                ],
            },
            RibbonGroup {
                label: "Finish".to_string(),
                has_dialog_launcher: false,
                items: vec![
                    RibbonItem::SplitButton {
                        id: "finish_merge".to_string(),
                        label: "Finish & Merge".to_string(),
                        icon: "mail-send-symbolic".to_string(),
                        items: vec![
                            DropdownItem { id: "finish_edit".to_string(), label: "Edit Individual Documents…".to_string(), icon: None, description: None },
                            DropdownItem { id: "finish_print".to_string(), label: "Print Documents…".to_string(), icon: None, description: None },
                            DropdownItem { id: "finish_email".to_string(), label: "Send Email Messages…".to_string(), icon: None, description: None },
                        ],
                    },
                ],
            },
        ],
    }
}

/// Return all default ribbon tabs in the canonical order.
pub fn all_tabs() -> Vec<RibbonTab> {
    vec![
        home_tab(),
        insert_tab(),
        design_tab(),
        layout_tab(),
        references_tab(),
        mailings_tab(),
        review_tab(),
        view_tab(),
    ]
}
