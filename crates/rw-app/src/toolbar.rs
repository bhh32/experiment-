//! Ribbon toolbar builders for each tab.

use cosmic::widget;
use cosmic::Element;

use crate::messages::Message;
use rw_ribbon::TabId;
use rw_widgets::status_bar::ViewMode;

/// Build the toolbar row for the given active tab.
pub fn build_toolbar(active_tab: TabId) -> Element<'static, Message> {
    let row = match active_tab {
        TabId::Home => build_home_toolbar(),
        TabId::Insert => build_insert_toolbar(),
        TabId::Design => build_design_toolbar(),
        TabId::Layout => build_layout_toolbar(),
        TabId::References => build_references_toolbar(),
        TabId::Mailings => build_mailings_toolbar(),
        TabId::Review => build_review_toolbar(),
        TabId::View => build_view_toolbar(),
        _ => widget::row().spacing(8)
            .push(widget::text::caption(format!("{:?} tab", active_tab))),
    };
    Element::from(row)
}

fn build_home_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Clipboard group
        .push(widget::button::text("Cut").on_press(Message::Cut))
        .push(widget::button::text("Copy").on_press(Message::Copy))
        .push(widget::button::text("Paste").on_press(Message::Paste(String::new())))
        .push(widget::text::body("|"))
        // Font formatting group
        .push(
            widget::button::text("B")
                .on_press(Message::RibbonAction("bold".into())),
        )
        .push(
            widget::button::text("I")
                .on_press(Message::RibbonAction("italic".into())),
        )
        .push(
            widget::button::text("U")
                .on_press(Message::RibbonAction("underline".into())),
        )
        .push(
            widget::button::text("S\u{0336}")
                .on_press(Message::RibbonAction("strikethrough".into())),
        )
        .push(widget::text::body("|"))
        // Paragraph formatting group
        .push(
            widget::button::icon(widget::icon::from_name("format-justify-left-symbolic"))
                .on_press(Message::RibbonAction("align_left".into())),
        )
        .push(
            widget::button::icon(widget::icon::from_name("format-justify-center-symbolic"))
                .on_press(Message::RibbonAction("align_center".into())),
        )
        .push(
            widget::button::icon(widget::icon::from_name("format-justify-right-symbolic"))
                .on_press(Message::RibbonAction("align_right".into())),
        )
        .push(
            widget::button::icon(widget::icon::from_name("format-justify-fill-symbolic"))
                .on_press(Message::RibbonAction("justify".into())),
        )
        .push(widget::text::body("|"))
        // Indent group
        .push(
            widget::button::text("Indent+")
                .on_press(Message::IncreaseIndent),
        )
        .push(
            widget::button::text("Indent-")
                .on_press(Message::DecreaseIndent),
        )
        .push(widget::text::body("|"))
        // List group
        .push(
            widget::button::icon(widget::icon::from_name("view-list-bullet-symbolic"))
                .on_press(Message::ToggleList(rw_editor::operations::ListType::Bullet)),
        )
        .push(
            widget::button::icon(widget::icon::from_name("view-list-ordered-symbolic"))
                .on_press(Message::ToggleList(rw_editor::operations::ListType::Numbered)),
        )
        .push(widget::text::body("|"))
        // Style group
        .push(
            widget::button::text("Normal")
                .on_press(Message::ApplyStyle("Normal".into())),
        )
        .push(
            widget::button::text("H1")
                .on_press(Message::ApplyStyle("Heading 1".into())),
        )
        .push(
            widget::button::text("H2")
                .on_press(Message::ApplyStyle("Heading 2".into())),
        )
        .push(
            widget::button::text("H3")
                .on_press(Message::ApplyStyle("Heading 3".into())),
        )
        .push(widget::text::body("|"))
        // Clear formatting
        .push(
            widget::button::text("Clear Fmt")
                .on_press(Message::ClearFormatting),
        )
}

fn build_insert_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Pages group
        .push(
            widget::button::text("Page Break")
                .on_press(Message::InsertPageBreak),
        )
        .push(widget::text::body("|"))
        // Tables group
        .push(
            widget::button::text("Table 2x2")
                .on_press(Message::InsertTable(2, 2)),
        )
        .push(
            widget::button::text("Table 3x3")
                .on_press(Message::InsertTable(3, 3)),
        )
        .push(
            widget::button::text("Table 4x4")
                .on_press(Message::InsertTable(4, 4)),
        )
        .push(widget::text::body("|"))
        // Illustrations group
        .push(
            widget::button::text("Image...")
                .on_press(Message::RibbonAction("insert_image".into())),
        )
        .push(widget::text::body("|"))
        // Links group
        .push(
            widget::button::text("Hyperlink")
                .on_press(Message::RibbonAction("insert_hyperlink".into())),
        )
        .push(
            widget::button::text("Bookmark")
                .on_press(Message::RibbonAction("insert_bookmark".into())),
        )
        .push(widget::text::body("|"))
        // Header & Footer group
        .push(
            widget::button::text("Page #")
                .on_press(Message::InsertPageNumber),
        )
        .push(
            widget::button::text("Date")
                .on_press(Message::InsertDate),
        )
        .push(widget::text::body("|"))
        // Symbols group
        .push(
            widget::button::text("Horiz. Rule")
                .on_press(Message::InsertHorizontalRule),
        )
        .push(
            widget::button::text("\u{2014}")
                .on_press(Message::InsertSymbol('\u{2014}')),
        )
        .push(
            widget::button::text("\u{00A9}")
                .on_press(Message::InsertSymbol('\u{00A9}')),
        )
        .push(
            widget::button::text("\u{2122}")
                .on_press(Message::InsertSymbol('\u{2122}')),
        )
}

fn build_design_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Themes group — style presets
        .push(widget::text::caption("Themes:"))
        .push(
            widget::button::text("Default")
                .on_press(Message::RibbonAction("theme_default".into())),
        )
        .push(
            widget::button::text("Professional")
                .on_press(Message::RibbonAction("theme_professional".into())),
        )
        .push(
            widget::button::text("Casual")
                .on_press(Message::RibbonAction("theme_casual".into())),
        )
        .push(widget::text::body("|"))
        // Page background group
        .push(widget::text::caption("Background:"))
        .push(
            widget::button::text("Page Color")
                .on_press(Message::RibbonAction("page_color".into())),
        )
        .push(
            widget::button::text("Page Borders")
                .on_press(Message::RibbonAction("page_borders".into())),
        )
        .push(
            widget::button::text("Watermark")
                .on_press(Message::RibbonAction("watermark".into())),
        )
}

fn build_layout_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Page setup group
        .push(widget::text::caption("Page Setup:"))
        .push(
            widget::button::text("Margins")
                .on_press(Message::RibbonAction("page_margins".into())),
        )
        .push(
            widget::button::text("Orientation")
                .on_press(Message::RibbonAction("page_orientation".into())),
        )
        .push(
            widget::button::text("Size")
                .on_press(Message::RibbonAction("page_size".into())),
        )
        .push(
            widget::button::text("Columns")
                .on_press(Message::RibbonAction("columns".into())),
        )
        .push(widget::text::body("|"))
        // Paragraph group
        .push(widget::text::caption("Paragraph:"))
        .push(
            widget::button::text("Indent+")
                .on_press(Message::IncreaseIndent),
        )
        .push(
            widget::button::text("Indent-")
                .on_press(Message::DecreaseIndent),
        )
        .push(
            widget::button::text("Spacing")
                .on_press(Message::RibbonAction("paragraph_spacing".into())),
        )
        .push(widget::text::body("|"))
        // Arrange group
        .push(widget::text::caption("Breaks:"))
        .push(
            widget::button::text("Page Break")
                .on_press(Message::InsertPageBreak),
        )
        .push(
            widget::button::text("Section Break")
                .on_press(Message::RibbonAction("section_break".into())),
        )
        .push(
            widget::button::text("Line Numbers")
                .on_press(Message::RibbonAction("line_numbers".into())),
        )
}

fn build_references_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Table of Contents group
        .push(widget::text::caption("Table of Contents:"))
        .push(
            widget::button::text("Insert TOC")
                .on_press(Message::RibbonAction("insert_toc".into())),
        )
        .push(
            widget::button::text("Update TOC")
                .on_press(Message::RibbonAction("update_toc".into())),
        )
        .push(widget::text::body("|"))
        // Footnotes group
        .push(widget::text::caption("Footnotes:"))
        .push(
            widget::button::text("Footnote")
                .on_press(Message::RibbonAction("insert_footnote".into())),
        )
        .push(
            widget::button::text("Endnote")
                .on_press(Message::RibbonAction("insert_endnote".into())),
        )
        .push(widget::text::body("|"))
        // Captions group
        .push(
            widget::button::text("Caption")
                .on_press(Message::RibbonAction("insert_caption".into())),
        )
        .push(
            widget::button::text("Cross-ref")
                .on_press(Message::RibbonAction("insert_cross_ref".into())),
        )
        .push(widget::text::body("|"))
        // Index group
        .push(
            widget::button::text("Mark Entry")
                .on_press(Message::RibbonAction("mark_index_entry".into())),
        )
        .push(
            widget::button::text("Insert Index")
                .on_press(Message::RibbonAction("insert_index".into())),
        )
        .push(widget::text::body("|"))
        // Bibliography group
        .push(
            widget::button::text("Bibliography")
                .on_press(Message::RibbonAction("insert_bibliography".into())),
        )
}

fn build_mailings_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Create group
        .push(widget::text::caption("Create:"))
        .push(
            widget::button::text("Envelopes")
                .on_press(Message::RibbonAction("envelopes".into())),
        )
        .push(
            widget::button::text("Labels")
                .on_press(Message::RibbonAction("labels".into())),
        )
        .push(widget::text::body("|"))
        // Start Mail Merge group
        .push(widget::text::caption("Mail Merge:"))
        .push(
            widget::button::text("Start Merge")
                .on_press(Message::RibbonAction("start_mail_merge".into())),
        )
        .push(
            widget::button::text("Select Recipients")
                .on_press(Message::RibbonAction("select_recipients".into())),
        )
        .push(widget::text::body("|"))
        // Write & Insert Fields group
        .push(widget::text::caption("Fields:"))
        .push(
            widget::button::text("Insert Merge Field")
                .on_press(Message::RibbonAction("insert_merge_field".into())),
        )
        .push(
            widget::button::text("Rules")
                .on_press(Message::RibbonAction("merge_rules".into())),
        )
        .push(widget::text::body("|"))
        // Preview & Finish group
        .push(
            widget::button::text("Preview Results")
                .on_press(Message::RibbonAction("preview_results".into())),
        )
        .push(
            widget::button::text("Finish & Merge")
                .on_press(Message::RibbonAction("finish_merge".into())),
        )
}

fn build_review_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Proofing group
        .push(widget::text::caption("Proofing:"))
        .push(
            widget::button::text("Spelling")
                .on_press(Message::ToggleSpellCheck),
        )
        .push(
            widget::button::text("Word Count")
                .on_press(Message::RibbonAction("word_count".into())),
        )
        .push(widget::text::body("|"))
        // Comments group
        .push(widget::text::caption("Comments:"))
        .push(
            widget::button::text("New Comment")
                .on_press(Message::RibbonAction("new_comment".into())),
        )
        .push(
            widget::button::text("Delete Comment")
                .on_press(Message::RibbonAction("delete_comment".into())),
        )
        .push(widget::text::body("|"))
        // Tracking group
        .push(widget::text::caption("Tracking:"))
        .push(
            widget::button::text("Track Changes")
                .on_press(Message::ToggleTrackChanges),
        )
        .push(widget::text::body("|"))
        // Changes group
        .push(widget::text::caption("Changes:"))
        .push(
            widget::button::text("Accept")
                .on_press(Message::AcceptChange),
        )
        .push(
            widget::button::text("Reject")
                .on_press(Message::RejectChange),
        )
        .push(
            widget::button::text("Accept All")
                .on_press(Message::AcceptAllChanges),
        )
        .push(
            widget::button::text("Reject All")
                .on_press(Message::RejectAllChanges),
        )
}

fn build_view_toolbar() -> widget::Row<'static, Message> {
    widget::row().spacing(4)
        // Views group
        .push(widget::text::caption("Views:"))
        .push(
            widget::button::text("Print Layout")
                .on_press(Message::ViewModeChanged(ViewMode::PrintLayout)),
        )
        .push(
            widget::button::text("Web Layout")
                .on_press(Message::ViewModeChanged(ViewMode::WebLayout)),
        )
        .push(
            widget::button::text("Outline")
                .on_press(Message::ViewModeChanged(ViewMode::Outline)),
        )
        .push(
            widget::button::text("Draft")
                .on_press(Message::ViewModeChanged(ViewMode::Draft)),
        )
        .push(
            widget::button::text("Read Mode")
                .on_press(Message::ViewModeChanged(ViewMode::ReadMode)),
        )
        .push(widget::text::body("|"))
        // Show group
        .push(widget::text::caption("Show:"))
        .push(
            widget::button::text("Ruler")
                .on_press(Message::ToggleRuler),
        )
        .push(
            widget::button::text("Formatting Marks")
                .on_press(Message::ToggleFormattingMarks),
        )
        .push(
            widget::button::text("Navigation Pane")
                .on_press(Message::ToggleNavigationPane),
        )
        .push(widget::text::body("|"))
        // Zoom group
        .push(widget::text::caption("Zoom:"))
        .push(
            widget::button::text("Zoom In")
                .on_press(Message::ZoomIn),
        )
        .push(
            widget::button::text("Zoom Out")
                .on_press(Message::ZoomOut),
        )
        .push(
            widget::button::text("100%")
                .on_press(Message::ZoomChanged(100)),
        )
        .push(widget::text::body("|"))
        // Window group
        .push(
            widget::button::text("Focus Mode")
                .on_press(Message::ToggleFocusMode),
        )
}
