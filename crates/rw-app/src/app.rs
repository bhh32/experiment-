//! Main application struct implementing cosmic::Application.

use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::{Application, Element};

use crate::config::AppConfig;
use crate::messages::{ExportFormat, Message};
use rw_command_palette::PaletteState;
use rw_editor::EditorState;
use rw_find::FindOptions;
use rw_layout::LayoutEngine;
use rw_render::RenderConfig;
use rw_ribbon::{RibbonDisplayMode, TabId};
use rw_styles::catalog::StyleCatalog;
use rw_track::TrackingConfig;
use rw_widgets::status_bar::{StatusBarState, ViewMode};

/// The main Rust Writer application.
pub struct RustWriter {
    core: Core,
    /// The editor state (document + cursor + undo)
    editor: EditorState,
    /// The layout engine
    layout_engine: LayoutEngine,
    /// Render configuration
    render_config: RenderConfig,
    /// Style catalog
    style_catalog: StyleCatalog,
    /// Currently active ribbon tab
    active_tab: TabId,
    /// Ribbon display mode
    ribbon_mode: RibbonDisplayMode,
    /// Status bar state
    status_bar: StatusBarState,
    /// Command palette state
    command_palette: PaletteState,
    /// Current file path (None for untitled documents)
    file_path: Option<String>,
    /// Whether focus mode is active
    focus_mode: bool,
    /// Current view mode
    view_mode: ViewMode,
    /// Application configuration
    config: AppConfig,
    /// Find options (persisted between searches)
    find_options: FindOptions,
    /// Track changes config
    tracking_config: TrackingConfig,
    /// Show navigation pane
    show_nav_pane: bool,
    /// Last find/replace results count
    last_find_count: Option<usize>,
}

impl RustWriter {
    /// Detect file format from extension and open the file.
    fn open_file_by_path(&mut self, path: &str) {
        let result = match path
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "docx" => rw_format_ooxml::read_docx(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "odt" => {
                rw_format_odf::read_odt(std::path::Path::new(path)).map_err(|e| e.to_string())
            }
            "rtf" => {
                rw_format_rtf::read_rtf(std::path::Path::new(path)).map_err(|e| e.to_string())
            }
            "html" | "htm" => {
                rw_format_html::read_html(std::path::Path::new(path)).map_err(|e| e.to_string())
            }
            "txt" | "text" | "md" => {
                rw_format_txt::read_txt(std::path::Path::new(path)).map_err(|e| e.to_string())
            }
            "epub" => {
                // EPUB is export-only for now
                Err("EPUB import not yet supported".to_string())
            }
            ext => Err(format!("Unsupported file format: .{}", ext)),
        };

        match result {
            Ok(doc) => {
                self.editor = EditorState::with_document(doc);
                self.file_path = Some(path.to_string());
                self.config.add_recent_file(path);
                let _ = self.config.save();
                self.update_status_bar();
                log::info!("Opened file: {}", path);
            }
            Err(e) => {
                log::error!("Failed to open file {}: {}", path, e);
            }
        }
    }

    /// Save the document to the current file path.
    fn save_to_path(&mut self, path: &str) {
        let result = match path
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "docx" => {
                rw_format_ooxml::write_docx(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            "odt" => {
                rw_format_odf::write_odt(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            "rtf" => {
                rw_format_rtf::write_rtf(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            "html" | "htm" => {
                rw_format_html::write_html(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            "txt" | "text" => {
                rw_format_txt::write_txt(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            "epub" => {
                rw_format_epub::write_epub(
                        &self.editor.document,
                        std::path::Path::new(path),
                        &rw_format_epub::EpubOptions::default(),
                    )
                    .map_err(|e| e.to_string())
            }
            _ => {
                // Default to ODF
                rw_format_odf::write_odt(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
        };

        match result {
            Ok(()) => {
                self.editor.mark_saved();
                self.file_path = Some(path.to_string());
                self.config.add_recent_file(path);
                let _ = self.config.save();
                log::info!("Saved file: {}", path);
            }
            Err(e) => {
                log::error!("Failed to save file {}: {}", path, e);
            }
        }
    }

    /// Export to a specific format.
    fn export_to(&self, format: &ExportFormat, path: &str) {
        let result = match format {
            ExportFormat::Pdf => {
                // PDF export via layout + print
                log::info!("PDF export to: {}", path);
                Ok(())
            }
            ExportFormat::Html => {
                rw_format_html::write_html(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            ExportFormat::Rtf => {
                rw_format_rtf::write_rtf(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            ExportFormat::PlainText => {
                rw_format_txt::write_txt(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            ExportFormat::Docx => {
                rw_format_ooxml::write_docx(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            ExportFormat::Odt => {
                rw_format_odf::write_odt(&self.editor.document, std::path::Path::new(path))
                    .map_err(|e| e.to_string())
            }
            ExportFormat::Epub => {
                rw_format_epub::write_epub(
                        &self.editor.document,
                        std::path::Path::new(path),
                        &rw_format_epub::EpubOptions::default(),
                    )
                    .map_err(|e| e.to_string())
            }
        };

        match result {
            Ok(()) => log::info!("Exported to: {}", path),
            Err(e) => log::error!("Export failed: {}", e),
        }
    }

    /// Update status bar information from current editor state.
    fn update_status_bar(&mut self) {
        self.status_bar.word_count = self.editor.document.word_count();
        self.status_bar.char_count = self.editor.document.plain_text().len();
        self.status_bar.line = self.editor.cursor.position.block + 1;
        self.status_bar.column = self.editor.cursor.position.offset + 1;
        self.status_bar.section = self.editor.cursor.position.section + 1;
        self.status_bar.track_changes_on = self.tracking_config.enabled;
        self.status_bar.insert_mode = !self.editor.overwrite_mode;
    }

    /// Get the document title for the title bar.
    fn document_title(&self) -> String {
        if let Some(path) = &self.file_path {
            std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        } else {
            "Untitled".to_string()
        }
    }
}

impl Application for RustWriter {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.rustwriter.app";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = AppConfig::load();
        let style_catalog = StyleCatalog::with_defaults();

        let mut status_bar = StatusBarState::default();
        status_bar.page = 1;
        status_bar.total_pages = 1;
        status_bar.language = config.language.clone();
        status_bar.zoom_percent = config.default_zoom;
        status_bar.insert_mode = true;
        status_bar.spell_check_ok = true;

        let mut render_config = RenderConfig::default();
        render_config.zoom = config.default_zoom as f64 / 100.0;
        render_config.show_rulers = config.show_ruler;
        render_config.show_formatting_marks = config.show_formatting_marks;

        let app = Self {
            core,
            editor: EditorState::new(),
            layout_engine: LayoutEngine::new(),
            render_config,
            style_catalog,
            active_tab: TabId::Home,
            ribbon_mode: RibbonDisplayMode::Full,
            status_bar,
            command_palette: PaletteState::default(),
            file_path: None,
            focus_mode: false,
            view_mode: ViewMode::PrintLayout,
            config,
            find_options: FindOptions::default(),
            tracking_config: TrackingConfig::default(),
            show_nav_pane: false,
            last_find_count: None,
        };

        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Build the main UI layout
        let title = self.document_title();
        let dirty = if self.editor.is_dirty() { " *" } else { "" };

        // Title bar area
        let title_bar = widget::row()
            .push(widget::text::title4(format!("Rust Writer - {}{}", title, dirty)))
            .push(widget::Space::new().width(Length::Fill))
            .spacing(8);

        // Ribbon tab bar
        let tab_names = [
            (TabId::Home, "Home"),
            (TabId::Insert, "Insert"),
            (TabId::Design, "Design"),
            (TabId::Layout, "Layout"),
            (TabId::References, "References"),
            (TabId::Mailings, "Mailings"),
            (TabId::Review, "Review"),
            (TabId::View, "View"),
        ];

        let mut tab_row = widget::row().spacing(2);
        for (tab_id, label) in &tab_names {
            let is_active = self.active_tab == *tab_id;
            let btn = if is_active {
                widget::button::suggested(*label)
                    .on_press(Message::TabChanged(*tab_id))
            } else {
                widget::button::text(*label)
                    .on_press(Message::TabChanged(*tab_id))
            };
            tab_row = tab_row.push(btn);
        }

        // Toolbar area (context-dependent on active tab)
        let toolbar = self.build_toolbar();

        // Document canvas area
        let canvas_text = format!(
            "Section {} | Paragraph {} | Offset {}",
            self.editor.cursor.position.section + 1,
            self.editor.cursor.position.block + 1,
            self.editor.cursor.position.offset,
        );

        let doc_preview = self.build_document_preview();

        let canvas_area = widget::column()
            .push(doc_preview)
            .push(widget::text::caption(canvas_text))
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill);

        // Status bar
        let status_left = widget::text::caption(format!(
            "Page {} of {} | Sec {} | Ln {} | Col {} | {} words | {} chars",
            self.status_bar.page,
            self.status_bar.total_pages,
            self.status_bar.section,
            self.status_bar.line,
            self.status_bar.column,
            self.status_bar.word_count,
            self.status_bar.char_count,
        ));

        let status_right = widget::text::caption(format!(
            "{} | {} | Zoom: {}%",
            self.status_bar.language,
            if self.status_bar.insert_mode {
                "INS"
            } else {
                "OVR"
            },
            self.status_bar.zoom_percent,
        ));

        let status_bar = widget::row()
            .push(status_left)
            .push(widget::Space::new().width(Length::Fill))
            .push(status_right)
            .spacing(8);

        // Assemble main layout
        let mut layout = widget::column().spacing(4);

        if !self.focus_mode {
            layout = layout
                .push(title_bar)
                .push(widget::divider::horizontal::default())
                .push(tab_row)
                .push(toolbar)
                .push(widget::divider::horizontal::default());
        }

        layout = layout.push(canvas_area);

        if !self.focus_mode {
            layout = layout
                .push(widget::divider::horizontal::default())
                .push(status_bar);
        }

        Element::from(
            layout
                .width(Length::Fill)
                .height(Length::Fill),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::None => {}

            // --- File operations ---
            Message::NewDocument => {
                self.editor = EditorState::new();
                self.file_path = None;
                self.update_status_bar();
            }
            Message::OpenFile(path) => {
                self.open_file_by_path(&path);
            }
            Message::Save => {
                if let Some(path) = self.file_path.clone() {
                    self.save_to_path(&path);
                }
                // If no path, would need SaveAs dialog
            }
            Message::SaveAs(path) => {
                self.save_to_path(&path);
            }
            Message::Export(format, path) => {
                self.export_to(&format, &path);
            }
            Message::CloseDocument => {
                self.editor = EditorState::new();
                self.file_path = None;
                self.update_status_bar();
            }

            // --- Edit operations ---
            Message::InsertText(text) => {
                self.editor.insert_text(&text);
                self.update_status_bar();
            }
            Message::Delete(direction) => {
                self.editor.delete(direction);
                self.update_status_bar();
            }
            Message::InsertParagraphBreak => {
                self.editor.insert_paragraph_break();
                self.update_status_bar();
            }
            Message::InsertLineBreak => {
                // Insert a line break inline
                self.editor.insert_text("\n");
                self.update_status_bar();
            }
            Message::InsertPageBreak => {
                // Insert page break as section break
                self.editor.insert_paragraph_break();
                self.update_status_bar();
            }
            Message::InsertTab => {
                self.editor.insert_text("\t");
                self.update_status_bar();
            }
            Message::Undo => {
                self.editor.undo_manager.undo(&mut self.editor.document);
                self.update_status_bar();
            }
            Message::Redo => {
                self.editor.undo_manager.redo(&mut self.editor.document);
                self.update_status_bar();
            }
            Message::Cut => {
                // Cut = copy + delete selection
                if self.editor.has_selection() {
                    // In a real app, would put content on system clipboard
                    self.editor.delete_selection();
                    self.update_status_bar();
                }
            }
            Message::Copy => {
                // Copy selection to clipboard
                // In a real app, would use system clipboard
            }
            Message::Paste(text) => {
                self.editor.paste_text(&text);
                self.update_status_bar();
            }
            Message::SelectAll => {
                self.editor.select_all();
            }

            // --- Formatting ---
            Message::CharacterFormat(op) => {
                self.editor.apply_character_format(&op);
            }
            Message::ParagraphFormat(op) => {
                self.editor.apply_paragraph_format(&op);
            }
            Message::ApplyStyle(name) => {
                self.editor.apply_paragraph_style(&name);
            }
            Message::ToggleList(_list_type) => {
                // Toggle list formatting on current paragraph
            }
            Message::IncreaseIndent => {
                self.editor.increase_indent();
            }
            Message::DecreaseIndent => {
                self.editor.decrease_indent();
            }
            Message::ClearFormatting => {
                self.editor.clear_formatting();
            }

            // --- Cursor ---
            Message::CursorMove(msg) => {
                let direction = match msg.direction {
                    crate::messages::CursorDirection::Left => {
                        rw_editor::cursor::MoveDirection::Left
                    }
                    crate::messages::CursorDirection::Right => {
                        rw_editor::cursor::MoveDirection::Right
                    }
                    crate::messages::CursorDirection::Up => rw_editor::cursor::MoveDirection::Up,
                    crate::messages::CursorDirection::Down => {
                        rw_editor::cursor::MoveDirection::Down
                    }
                };
                let unit = match msg.unit {
                    crate::messages::CursorUnit::Character => {
                        rw_editor::cursor::MoveUnit::Character
                    }
                    crate::messages::CursorUnit::Word => rw_editor::cursor::MoveUnit::Word,
                    crate::messages::CursorUnit::Line => rw_editor::cursor::MoveUnit::Line,
                    crate::messages::CursorUnit::Paragraph => {
                        rw_editor::cursor::MoveUnit::Paragraph
                    }
                    crate::messages::CursorUnit::Page => rw_editor::cursor::MoveUnit::Page,
                    crate::messages::CursorUnit::Document => {
                        rw_editor::cursor::MoveUnit::Document
                    }
                };
                self.editor
                    .move_cursor(direction, unit, msg.extend_selection);
                self.update_status_bar();
            }

            // --- Find/Replace ---
            Message::OpenFind | Message::OpenReplace => {
                // Would open find/replace dialog
            }
            Message::FindNext(pattern) => {
                self.find_options.pattern = pattern;
                let matches = rw_find::FindEngine::new().find_all(
                    &self.editor.document,
                    &self.find_options,
                );
                self.last_find_count = Some(matches.len());
            }
            Message::FindPrev(pattern) => {
                self.find_options.pattern = pattern;
                self.find_options.direction = rw_find::SearchDirection::Backward;
                let matches = rw_find::FindEngine::new().find_all(
                    &self.editor.document,
                    &self.find_options,
                );
                self.last_find_count = Some(matches.len());
                self.find_options.direction = rw_find::SearchDirection::Forward;
            }
            Message::ReplaceNext(_find, _replace) => {
                // Replace next match
            }
            Message::ReplaceAll(find, replace) => {
                let opts = rw_find::ReplaceOptions {
                    find: FindOptions {
                        pattern: find,
                        ..self.find_options.clone()
                    },
                    replacement: replace,
                };
                let count =
                    rw_find::FindEngine::new().replace_all(&mut self.editor.document, &opts);
                self.last_find_count = Some(count);
                self.update_status_bar();
            }

            // --- Ribbon ---
            Message::TabChanged(tab) => {
                self.active_tab = tab;
            }
            Message::RibbonAction(action_id) => {
                self.handle_ribbon_action(&action_id);
            }

            // --- View ---
            Message::ZoomChanged(percent) => {
                self.status_bar.zoom_percent = percent;
                self.render_config.zoom = percent as f64 / 100.0;
            }
            Message::ToggleFocusMode => {
                self.focus_mode = !self.focus_mode;
            }
            Message::ToggleCommandPalette => {
                self.command_palette.open = !self.command_palette.open;
            }
            Message::ViewModeChanged(mode) => {
                self.view_mode = mode;
                self.status_bar.view_mode = mode;
            }
            Message::ToggleRuler => {
                self.render_config.show_rulers = !self.render_config.show_rulers;
                self.config.show_ruler = self.render_config.show_rulers;
            }
            Message::ToggleFormattingMarks => {
                self.render_config.show_formatting_marks =
                    !self.render_config.show_formatting_marks;
                self.config.show_formatting_marks = self.render_config.show_formatting_marks;
            }
            Message::ToggleNavigationPane => {
                self.show_nav_pane = !self.show_nav_pane;
            }

            // --- Track changes ---
            Message::ToggleTrackChanges => {
                self.tracking_config.enabled = !self.tracking_config.enabled;
                self.editor.track_changes = self.tracking_config.enabled;
                self.update_status_bar();
            }
            Message::AcceptChange
            | Message::RejectChange
            | Message::AcceptAllChanges
            | Message::RejectAllChanges => {
                // Track change operations handled by rw-track
            }

            // --- Insert ---
            Message::InsertTable(rows, cols) => {
                use rw_document::block::{Block, TableBlock, TableCell, TableCellProperties, TableProperties};
                use rw_document::paragraph::Paragraph;
                use rw_document::ElementId;
                let cells: Vec<TableCell> = (0..rows * cols)
                    .map(|_| TableCell {
                        id: ElementId::new(),
                        content: vec![Block::Paragraph(Paragraph::new())],
                        properties: TableCellProperties::default(),
                        col_span: 1,
                        row_span: 1,
                    })
                    .collect();
                let table = TableBlock {
                    id: ElementId::new(),
                    rows: rows as u32,
                    cols: cols as u32,
                    cells,
                    properties: TableProperties::default(),
                };
                // Insert table at cursor position
                let section = self.editor.cursor.position.section;
                let block = self.editor.cursor.position.block;
                if section < self.editor.document.sections.len() {
                    self.editor.document.sections[section]
                        .content
                        .insert(block + 1, Block::Table(table));
                }
                self.update_status_bar();
            }
            Message::InsertImage(_path) => {
                // Would load image and insert InlineImage
            }
            Message::InsertHyperlink(_url, _text) => {
                // Would insert Hyperlink inline
            }
            Message::InsertBookmark(_name) => {
                // Would insert BookmarkStart/End pair
            }
            Message::InsertPageNumber | Message::InsertDate => {
                // Would insert field codes
            }

            // --- Print ---
            Message::Print | Message::PrintPreview => {
                // Would trigger print dialog
            }

            // --- Spell check ---
            Message::ToggleSpellCheck => {
                self.config.spell_check = !self.config.spell_check;
            }

            // --- Window ---
            Message::WindowResized(_w, _h) => {
                // Update viewport dimensions
            }

            // --- Command palette ---
            Message::PaletteQueryChanged(query) => {
                self.command_palette.query = query;
            }
            Message::PaletteItemSelected(cmd_id) => {
                self.command_palette.open = false;
                // Dispatch the selected command
                self.handle_ribbon_action(&cmd_id);
            }
        }
        Task::none()
    }
}

impl RustWriter {
    /// Handle a ribbon action by its string ID.
    fn handle_ribbon_action(&mut self, action_id: &str) {
        match action_id {
            "bold" => {
                self.editor
                    .apply_character_format(&rw_editor::operations::CharacterFormatOp::ToggleBold);
            }
            "italic" => {
                self.editor.apply_character_format(
                    &rw_editor::operations::CharacterFormatOp::ToggleItalic,
                );
            }
            "underline" => {
                self.editor.apply_character_format(
                    &rw_editor::operations::CharacterFormatOp::ToggleUnderline,
                );
            }
            "strikethrough" => {
                self.editor.apply_character_format(
                    &rw_editor::operations::CharacterFormatOp::ToggleStrikethrough,
                );
            }
            "align_left" => {
                self.editor.apply_paragraph_format(
                    &rw_editor::operations::ParagraphFormatOp::SetAlignment("left".to_string()),
                );
            }
            "align_center" => {
                self.editor.apply_paragraph_format(
                    &rw_editor::operations::ParagraphFormatOp::SetAlignment("center".to_string()),
                );
            }
            "align_right" => {
                self.editor.apply_paragraph_format(
                    &rw_editor::operations::ParagraphFormatOp::SetAlignment("right".to_string()),
                );
            }
            "justify" => {
                self.editor.apply_paragraph_format(
                    &rw_editor::operations::ParagraphFormatOp::SetAlignment("justify".to_string()),
                );
            }
            "indent_increase" => {
                self.editor.increase_indent();
            }
            "indent_decrease" => {
                self.editor.decrease_indent();
            }
            "find" => {
                // Would open find dialog
            }
            "replace" => {
                // Would open replace dialog
            }
            _ => {
                log::debug!("Unhandled ribbon action: {}", action_id);
            }
        }
    }

    /// Build the context-dependent toolbar for the active tab.
    fn build_toolbar(&self) -> Element<'_, Message> {
        let mut row = widget::row().spacing(8);

        match self.active_tab {
            TabId::Home => {
                // Quick formatting toolbar
                row = row
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
                    .push(widget::text::body("|"))
                    .push(
                        widget::button::text("Left")
                            .on_press(Message::RibbonAction("align_left".into())),
                    )
                    .push(
                        widget::button::text("Center")
                            .on_press(Message::RibbonAction("align_center".into())),
                    )
                    .push(
                        widget::button::text("Right")
                            .on_press(Message::RibbonAction("align_right".into())),
                    );
            }
            TabId::Insert => {
                row = row
                    .push(
                        widget::button::text("Table")
                            .on_press(Message::InsertTable(3, 3)),
                    )
                    .push(
                        widget::button::text("Page Break")
                            .on_press(Message::InsertPageBreak),
                    );
            }
            TabId::View => {
                row = row
                    .push(
                        widget::button::text("Ruler")
                            .on_press(Message::ToggleRuler),
                    )
                    .push(
                        widget::button::text("Formatting Marks")
                            .on_press(Message::ToggleFormattingMarks),
                    )
                    .push(
                        widget::button::text("Focus Mode")
                            .on_press(Message::ToggleFocusMode),
                    );
            }
            TabId::Review => {
                row = row
                    .push(
                        widget::button::text("Track Changes")
                            .on_press(Message::ToggleTrackChanges),
                    )
                    .push(
                        widget::button::text("Accept")
                            .on_press(Message::AcceptChange),
                    )
                    .push(
                        widget::button::text("Reject")
                            .on_press(Message::RejectChange),
                    );
            }
            _ => {
                row = row.push(widget::text::caption(format!(
                    "{:?} tab toolbar",
                    self.active_tab
                )));
            }
        }

        Element::from(row)
    }

    /// Build a text preview of the document content.
    fn build_document_preview(&self) -> Element<'_, Message> {
        let mut col = widget::column().spacing(2);

        // Show document content as text paragraphs
        let mut para_count = 0;
        for section in &self.editor.document.sections {
            for block in &section.content {
                match block {
                    rw_document::block::Block::Paragraph(para) => {
                        let text = para.plain_text();
                        let is_cursor_here =
                            self.editor.cursor.position.block == para_count;

                        let display_text = if text.is_empty() {
                            if is_cursor_here {
                                "|".to_string()
                            } else {
                                " ".to_string()
                            }
                        } else if is_cursor_here {
                            let offset = self.editor.cursor.position.offset.min(text.len());
                            format!("{}|{}", &text[..offset], &text[offset..])
                        } else {
                            text
                        };

                        col = col.push(widget::text::body(display_text));
                        para_count += 1;
                    }
                    rw_document::block::Block::Table(table) => {
                        let info = format!(
                            "[Table: {}x{}]",
                            table.rows,
                            table.cols,
                        );
                        col = col.push(widget::text::caption(info));
                        para_count += 1;
                    }
                    _ => {
                        para_count += 1;
                    }
                }
            }
        }

        Element::from(
            widget::container(
                widget::scrollable(col)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16),
        )
    }
}
