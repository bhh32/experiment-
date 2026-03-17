//! Main application struct implementing cosmic::Application.

use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::{Application, Element};

use crate::config::AppConfig;
use crate::messages::{BackstagePage, ExportFormat, KeyCode, KeyModifiers, Message};
use crate::toolbar;
use rw_command_palette::{CommandRegistry, PaletteState};
use rw_editor::EditorState;
use rw_find::FindOptions;
use rw_layout::LayoutEngine;
use rw_render::RenderConfig;
use rw_ribbon::{RibbonDisplayMode, TabId};
use rw_styles::catalog::StyleCatalog;
use rw_track::TrackingConfig;
use rw_widgets::status_bar::{StatusBarState, ViewMode};

/// The main Rust Writer application.
#[allow(dead_code)]
pub struct RustWriter {
    core: Core,
    editor: EditorState,
    layout_engine: LayoutEngine,
    render_config: RenderConfig,
    style_catalog: StyleCatalog,
    active_tab: TabId,
    ribbon_mode: RibbonDisplayMode,
    status_bar: StatusBarState,
    command_palette: PaletteState,
    command_registry: CommandRegistry,
    file_path: Option<String>,
    focus_mode: bool,
    view_mode: ViewMode,
    config: AppConfig,
    find_options: FindOptions,
    tracking_config: TrackingConfig,
    show_nav_pane: bool,
    last_find_count: Option<usize>,
    backstage_open: bool,
    backstage_page: BackstagePage,
    show_find_bar: bool,
    find_query: String,
    replace_query: String,
}

// ---------------------------------------------------------------------------
// File I/O helpers
// ---------------------------------------------------------------------------

impl RustWriter {
    fn open_file_by_path(&mut self, path: &str) {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        let result = match ext.as_str() {
            "docx" => rw_format_ooxml::read_docx(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "odt" => rw_format_odf::read_odt(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "rtf" => rw_format_rtf::read_rtf(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "html" | "htm" => rw_format_html::read_html(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "txt" | "text" | "md" => rw_format_txt::read_txt(std::path::Path::new(path))
                .map_err(|e| e.to_string()),
            "epub" => Err("EPUB import not yet supported".to_string()),
            other => Err(format!("Unsupported file format: .{}", other)),
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
            Err(e) => log::error!("Failed to open file {}: {}", path, e),
        }
    }

    fn save_to_path(&mut self, path: &str) {
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        let result = match ext.as_str() {
            "docx" => rw_format_ooxml::write_docx(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            "odt" => rw_format_odf::write_odt(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            "rtf" => rw_format_rtf::write_rtf(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            "html" | "htm" => rw_format_html::write_html(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            "txt" | "text" => rw_format_txt::write_txt(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            "epub" => rw_format_epub::write_epub(
                &self.editor.document, std::path::Path::new(path),
                &rw_format_epub::EpubOptions::default(),
            ).map_err(|e| e.to_string()),
            _ => rw_format_odf::write_odt(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
        };

        match result {
            Ok(()) => {
                self.editor.mark_saved();
                self.file_path = Some(path.to_string());
                self.config.add_recent_file(path);
                let _ = self.config.save();
                log::info!("Saved file: {}", path);
            }
            Err(e) => log::error!("Failed to save file {}: {}", path, e),
        }
    }

    fn export_to(&self, format: &ExportFormat, path: &str) {
        let result = match format {
            ExportFormat::Pdf => { log::info!("PDF export to: {}", path); Ok(()) }
            ExportFormat::Html => rw_format_html::write_html(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            ExportFormat::Rtf => rw_format_rtf::write_rtf(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            ExportFormat::PlainText => rw_format_txt::write_txt(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            ExportFormat::Docx => rw_format_ooxml::write_docx(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            ExportFormat::Odt => rw_format_odf::write_odt(
                &self.editor.document, std::path::Path::new(path),
            ).map_err(|e| e.to_string()),
            ExportFormat::Epub => rw_format_epub::write_epub(
                &self.editor.document, std::path::Path::new(path),
                &rw_format_epub::EpubOptions::default(),
            ).map_err(|e| e.to_string()),
        };

        match result {
            Ok(()) => log::info!("Exported to: {}", path),
            Err(e) => log::error!("Export failed: {}", e),
        }
    }

    fn update_status_bar(&mut self) {
        self.status_bar.word_count = self.editor.document.word_count();
        self.status_bar.char_count = self.editor.document.plain_text().len();
        self.status_bar.line = self.editor.cursor.position.block + 1;
        self.status_bar.column = self.editor.cursor.position.offset + 1;
        self.status_bar.section = self.editor.cursor.position.section + 1;
        self.status_bar.track_changes_on = self.tracking_config.enabled;
        self.status_bar.insert_mode = !self.editor.overwrite_mode;
    }

    fn document_title(&self) -> String {
        self.file_path.as_ref().map_or_else(
            || "Untitled".to_string(),
            |path| {
                std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string())
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Application trait
// ---------------------------------------------------------------------------

impl Application for RustWriter {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.rustwriter.app";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

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
            command_registry: CommandRegistry::with_defaults(),
            file_path: None,
            focus_mode: false,
            view_mode: ViewMode::PrintLayout,
            config,
            find_options: FindOptions::default(),
            tracking_config: TrackingConfig::default(),
            show_nav_pane: false,
            last_find_count: None,
            backstage_open: false,
            backstage_page: BackstagePage::Info,
            show_find_bar: false,
            find_query: String::new(),
            replace_query: String::new(),
        };

        (app, Task::none())
    }

    // --- COSMIC header bar ---

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let file_btn = widget::button::text("File")
            .on_press(Message::OpenBackstage);
        let title = self.document_title();
        let dirty = if self.editor.is_dirty() { " *" } else { "" };
        let title_label = widget::text::body(format!("{}{}", title, dirty));
        vec![file_btn.into(), title_label.into()]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let undo_btn = widget::button::icon(widget::icon::from_name("edit-undo-symbolic"))
            .on_press(Message::Undo);
        let redo_btn = widget::button::icon(widget::icon::from_name("edit-redo-symbolic"))
            .on_press(Message::Redo);
        let palette_btn = widget::button::icon(widget::icon::from_name("system-search-symbolic"))
            .on_press(Message::ToggleCommandPalette);
        vec![undo_btn.into(), redo_btn.into(), palette_btn.into()]
    }

    // --- View ---

    fn view(&self) -> Element<'_, Self::Message> {
        // If backstage is open, show the backstage view instead of the document
        if self.backstage_open {
            return self.build_backstage_view();
        }

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
            let btn = if self.active_tab == *tab_id {
                widget::button::suggested(*label)
                    .on_press(Message::TabChanged(*tab_id))
            } else {
                widget::button::text(*label)
                    .on_press(Message::TabChanged(*tab_id))
            };
            tab_row = tab_row.push(btn);
        }

        // Toolbar (delegated to toolbar module)
        let active_toolbar = toolbar::build_toolbar(self.active_tab);

        // Document content area
        let doc_content = self.build_document_preview();

        // Optional find/replace bar
        let find_bar = if self.show_find_bar {
            Some(self.build_find_bar())
        } else {
            None
        };

        // Status bar
        let status_left = widget::text::caption(format!(
            "Page {} of {} | Sec {} | Ln {} | Col {} | {} words | {} chars",
            self.status_bar.page, self.status_bar.total_pages,
            self.status_bar.section, self.status_bar.line,
            self.status_bar.column, self.status_bar.word_count,
            self.status_bar.char_count,
        ));
        let status_center = widget::text::caption(format!("{:?}", self.view_mode));
        let status_right = widget::text::caption(format!(
            "{} | {} | Zoom: {}%",
            self.status_bar.language,
            if self.status_bar.insert_mode { "INS" } else { "OVR" },
            self.status_bar.zoom_percent,
        ));
        let status_row = widget::row().spacing(8)
            .push(status_left)
            .push(widget::Space::new().width(Length::Fill))
            .push(status_center)
            .push(widget::Space::new().width(Length::Fill))
            .push(status_right);

        // Assemble layout
        let mut layout = widget::column().spacing(2);

        if !self.focus_mode {
            layout = layout
                .push(tab_row)
                .push(active_toolbar)
                .push(widget::divider::horizontal::default());
        }

        // Optional navigation pane + document
        if self.show_nav_pane {
            let nav_pane = self.build_navigation_pane();
            let body = widget::row()
                .push(nav_pane)
                .push(widget::divider::vertical::default())
                .push(doc_content)
                .width(Length::Fill)
                .height(Length::Fill);
            layout = layout.push(body);
        } else {
            layout = layout.push(doc_content);
        }

        if let Some(fb) = find_bar {
            layout = layout.push(widget::divider::horizontal::default()).push(fb);
        }

        if !self.focus_mode {
            layout = layout
                .push(widget::divider::horizontal::default())
                .push(status_row);
        }

        // Command palette: show above the main layout if open
        if self.command_palette.open {
            let palette = self.build_command_palette();
            layout = layout.push(widget::divider::horizontal::default()).push(palette);
        }

        Element::from(layout.width(Length::Fill).height(Length::Fill))
    }

    // --- Update ---

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::None => {}

            // --- File ---
            Message::NewDocument => {
                self.editor = EditorState::new();
                self.file_path = None;
                self.backstage_open = false;
                self.update_status_bar();
            }
            Message::RequestOpenFile => {
                // In production, this would spawn a native file dialog.
                // For now, close backstage and log.
                self.backstage_open = false;
                log::info!("Open file dialog requested");
            }
            Message::OpenFile(path) => {
                self.open_file_by_path(&path);
                self.backstage_open = false;
            }
            Message::Save => {
                if let Some(path) = self.file_path.clone() {
                    self.save_to_path(&path);
                } else {
                    log::info!("Save As dialog needed (no file path)");
                }
            }
            Message::RequestSaveAs => {
                log::info!("Save As dialog requested");
            }
            Message::SaveAs(path) => {
                self.save_to_path(&path);
                self.backstage_open = false;
            }
            Message::Export(format, path) => {
                self.export_to(&format, &path);
            }
            Message::CloseDocument => {
                self.editor = EditorState::new();
                self.file_path = None;
                self.backstage_open = false;
                self.update_status_bar();
            }

            // --- Edit ---
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
                self.editor.apply_edit_operation(
                    &rw_editor::operations::EditOperation::InsertLineBreak,
                );
                self.update_status_bar();
            }
            Message::InsertPageBreak => {
                self.editor.apply_edit_operation(
                    &rw_editor::operations::EditOperation::InsertPageBreak,
                );
                self.update_status_bar();
            }
            Message::InsertTab => {
                self.editor.apply_edit_operation(
                    &rw_editor::operations::EditOperation::InsertTab,
                );
                self.update_status_bar();
            }
            Message::Undo => {
                self.editor.undo();
                self.update_status_bar();
            }
            Message::Redo => {
                self.editor.redo();
                self.update_status_bar();
            }
            Message::Cut => {
                if self.editor.has_selection() {
                    let _content = self.editor.get_clipboard_content();
                    // In production, copy to system clipboard
                    self.editor.delete_selection();
                    self.update_status_bar();
                }
            }
            Message::Copy => {
                let _content = self.editor.get_clipboard_content();
                // In production, copy to system clipboard
            }
            Message::Paste(text) => {
                if !text.is_empty() {
                    self.editor.paste_text(&text);
                    self.update_status_bar();
                }
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
                log::debug!("Toggle list: not yet implemented");
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
                    crate::messages::CursorDirection::Left => rw_editor::cursor::MoveDirection::Left,
                    crate::messages::CursorDirection::Right => rw_editor::cursor::MoveDirection::Right,
                    crate::messages::CursorDirection::Up => rw_editor::cursor::MoveDirection::Up,
                    crate::messages::CursorDirection::Down => rw_editor::cursor::MoveDirection::Down,
                };
                let unit = match msg.unit {
                    crate::messages::CursorUnit::Character => rw_editor::cursor::MoveUnit::Character,
                    crate::messages::CursorUnit::Word => rw_editor::cursor::MoveUnit::Word,
                    crate::messages::CursorUnit::Line => rw_editor::cursor::MoveUnit::Line,
                    crate::messages::CursorUnit::Paragraph => rw_editor::cursor::MoveUnit::Paragraph,
                    crate::messages::CursorUnit::Page => rw_editor::cursor::MoveUnit::Page,
                    crate::messages::CursorUnit::Document => rw_editor::cursor::MoveUnit::Document,
                };
                self.editor.move_cursor(direction, unit, msg.extend_selection);
                self.update_status_bar();
            }

            // --- Find/Replace ---
            Message::OpenFind => {
                self.show_find_bar = true;
            }
            Message::OpenReplace => {
                self.show_find_bar = true;
            }
            Message::CloseFindReplace => {
                self.show_find_bar = false;
            }
            Message::FindNext(pattern) => {
                self.find_query = pattern.clone();
                self.find_options.pattern = pattern;
                let matches = rw_find::FindEngine::new().find_all(
                    &self.editor.document, &self.find_options,
                );
                self.last_find_count = Some(matches.len());
            }
            Message::FindPrev(pattern) => {
                self.find_query = pattern.clone();
                self.find_options.pattern = pattern;
                self.find_options.direction = rw_find::SearchDirection::Backward;
                let matches = rw_find::FindEngine::new().find_all(
                    &self.editor.document, &self.find_options,
                );
                self.last_find_count = Some(matches.len());
                self.find_options.direction = rw_find::SearchDirection::Forward;
            }
            Message::ReplaceNext(find, replace) => {
                log::debug!("Replace next: '{}' -> '{}'", find, replace);
            }
            Message::ReplaceAll(find, replace) => {
                let opts = rw_find::ReplaceOptions {
                    find: FindOptions {
                        pattern: find,
                        ..self.find_options.clone()
                    },
                    replacement: replace,
                };
                let count = rw_find::FindEngine::new()
                    .replace_all(&mut self.editor.document, &opts);
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
            Message::ZoomIn => {
                let new = (self.status_bar.zoom_percent + 10).min(500);
                self.status_bar.zoom_percent = new;
                self.render_config.zoom = new as f64 / 100.0;
            }
            Message::ZoomOut => {
                let new = self.status_bar.zoom_percent.saturating_sub(10).max(10);
                self.status_bar.zoom_percent = new;
                self.render_config.zoom = new as f64 / 100.0;
            }
            Message::ToggleFocusMode => {
                self.focus_mode = !self.focus_mode;
            }
            Message::ToggleCommandPalette => {
                self.command_palette.open = !self.command_palette.open;
                if self.command_palette.open {
                    self.command_palette.query.clear();
                    self.command_palette.selected = 0;
                }
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
                self.render_config.show_formatting_marks = !self.render_config.show_formatting_marks;
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
            Message::AcceptChange => {
                log::debug!("Accept change: not yet connected to rw-track");
            }
            Message::RejectChange => {
                log::debug!("Reject change: not yet connected to rw-track");
            }
            Message::AcceptAllChanges => {
                log::debug!("Accept all changes: not yet connected to rw-track");
            }
            Message::RejectAllChanges => {
                log::debug!("Reject all changes: not yet connected to rw-track");
            }

            // --- Insert ---
            Message::InsertTable(rows, cols) => {
                self.insert_table(rows, cols);
            }
            Message::InsertImage(path) => {
                self.insert_image_at_cursor(&path);
            }
            Message::InsertHyperlink(url, text) => {
                self.insert_hyperlink_at_cursor(&url, &text);
            }
            Message::InsertBookmark(name) => {
                self.insert_bookmark_at_cursor(&name);
            }
            Message::InsertPageNumber => {
                self.insert_field_at_cursor(rw_document::inline::FieldType::PageNumber);
            }
            Message::InsertDate => {
                self.insert_field_at_cursor(rw_document::inline::FieldType::Date {
                    format: "%Y-%m-%d".to_string(),
                });
            }
            Message::InsertHorizontalRule => {
                use rw_document::block::HorizontalRule;
                let section = self.editor.cursor.position.section;
                let block = self.editor.cursor.position.block;
                if section < self.editor.document.sections.len() {
                    self.editor.document.sections[section]
                        .content
                        .insert(block + 1, rw_document::Block::HorizontalRule(HorizontalRule {
                            id: rw_document::ElementId::new(),
                            width_percent: 100.0,
                            height: rw_document::Twips::from_points(1.0),
                            color: rw_document::Color::BLACK,
                            alignment: rw_document::properties::Alignment::Center,
                        }));
                }
                self.update_status_bar();
            }
            Message::InsertSymbol(ch) => {
                self.editor.insert_text(&ch.to_string());
                self.update_status_bar();
            }

            // --- Print ---
            Message::Print => {
                log::info!("Print requested");
            }
            Message::PrintPreview => {
                log::info!("Print preview requested");
            }

            // --- Spell check ---
            Message::ToggleSpellCheck => {
                self.config.spell_check = !self.config.spell_check;
            }

            // --- Window ---
            Message::WindowResized(_w, _h) => {}

            // --- Command palette ---
            Message::PaletteQueryChanged(query) => {
                self.command_palette.query = query;
            }
            Message::PaletteItemSelected(cmd_id) => {
                self.command_palette.open = false;
                self.command_registry.record_use(&cmd_id);
                self.handle_ribbon_action(&cmd_id);
            }

            // --- Backstage ---
            Message::OpenBackstage => {
                self.backstage_open = true;
                self.backstage_page = BackstagePage::Info;
            }
            Message::CloseBackstage => {
                self.backstage_open = false;
            }
            Message::BackstageNavigate(page) => {
                self.backstage_page = page;
                // Handle immediate actions
                match page {
                    BackstagePage::New => {
                        return Task::done(cosmic::Action::App(Message::NewDocument));
                    }
                    BackstagePage::Open => {
                        return Task::done(cosmic::Action::App(Message::RequestOpenFile));
                    }
                    BackstagePage::Save => {
                        return Task::done(cosmic::Action::App(Message::Save));
                    }
                    _ => {}
                }
            }

            // --- Keyboard ---
            Message::KeyPressed(key, mods) => {
                if let Some(msg) = crate::keyboard::map_key_to_message(key, mods) {
                    return Task::done(cosmic::Action::App(msg));
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        use cosmic::iced::keyboard::Event as KbEvent;
        use cosmic::iced::keyboard::Key;
        use cosmic::iced::keyboard::key::Named;

        cosmic::iced_futures::subscription::filter_map(
            std::any::TypeId::of::<Self>(),
            move |event| {
                if let cosmic::iced_futures::subscription::Event::Interaction {
                    event: cosmic::iced::Event::Keyboard(kb_event),
                    ..
                } = event
                {
                    match kb_event {
                        KbEvent::KeyPressed { key, modifiers, text, .. } => {
                            let mods = KeyModifiers {
                                shift: modifiers.shift(),
                                ctrl: modifiers.control(),
                                alt: modifiers.alt(),
                                logo: modifiers.logo(),
                            };
                            let code = match key {
                                Key::Named(named) => match named {
                                    Named::Enter => Some(KeyCode::Enter),
                                    Named::Tab => Some(KeyCode::Tab),
                                    Named::Backspace => Some(KeyCode::Backspace),
                                    Named::Delete => Some(KeyCode::Delete),
                                    Named::Escape => Some(KeyCode::Escape),
                                    Named::ArrowLeft => Some(KeyCode::Left),
                                    Named::ArrowRight => Some(KeyCode::Right),
                                    Named::ArrowUp => Some(KeyCode::Up),
                                    Named::ArrowDown => Some(KeyCode::Down),
                                    Named::Home => Some(KeyCode::Home),
                                    Named::End => Some(KeyCode::End),
                                    Named::PageUp => Some(KeyCode::PageUp),
                                    Named::PageDown => Some(KeyCode::PageDown),
                                    Named::Insert => Some(KeyCode::Insert),
                                    Named::F1 => Some(KeyCode::F1),
                                    Named::F2 => Some(KeyCode::F2),
                                    Named::F3 => Some(KeyCode::F3),
                                    Named::F4 => Some(KeyCode::F4),
                                    Named::F5 => Some(KeyCode::F5),
                                    Named::F6 => Some(KeyCode::F6),
                                    Named::F7 => Some(KeyCode::F7),
                                    Named::F8 => Some(KeyCode::F8),
                                    Named::F9 => Some(KeyCode::F9),
                                    Named::F10 => Some(KeyCode::F10),
                                    Named::F11 => Some(KeyCode::F11),
                                    Named::F12 => Some(KeyCode::F12),
                                    _ => None,
                                },
                                Key::Character(ref c) => c.chars().next().map(KeyCode::Character),
                                _ => text.and_then(|t| t.chars().next()).map(KeyCode::Character),
                            };
                            code.map(|k| Message::KeyPressed(k, mods))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Ribbon action dispatch
// ---------------------------------------------------------------------------

impl RustWriter {
    fn handle_ribbon_action(&mut self, action_id: &str) {
        use rw_editor::operations::{CharacterFormatOp, ParagraphFormatOp};
        match action_id {
            "bold" => self.editor.apply_character_format(&CharacterFormatOp::ToggleBold),
            "italic" => self.editor.apply_character_format(&CharacterFormatOp::ToggleItalic),
            "underline" => self.editor.apply_character_format(&CharacterFormatOp::ToggleUnderline),
            "strikethrough" => self.editor.apply_character_format(&CharacterFormatOp::ToggleStrikethrough),
            "superscript" => self.editor.apply_character_format(&CharacterFormatOp::ToggleSuperscript),
            "subscript" => self.editor.apply_character_format(&CharacterFormatOp::ToggleSubscript),
            "align_left" => self.editor.apply_paragraph_format(&ParagraphFormatOp::SetAlignment("left".to_string())),
            "align_center" => self.editor.apply_paragraph_format(&ParagraphFormatOp::SetAlignment("center".to_string())),
            "align_right" => self.editor.apply_paragraph_format(&ParagraphFormatOp::SetAlignment("right".to_string())),
            "justify" => self.editor.apply_paragraph_format(&ParagraphFormatOp::SetAlignment("justify".to_string())),
            "indent_increase" => self.editor.increase_indent(),
            "indent_decrease" => self.editor.decrease_indent(),

            // Command palette mapped IDs
            "file.new" => { self.editor = EditorState::new(); self.file_path = None; self.update_status_bar(); }
            "file.save" => { if let Some(p) = self.file_path.clone() { self.save_to_path(&p); } }
            "file.close" => { self.editor = EditorState::new(); self.file_path = None; self.update_status_bar(); }
            "edit.undo" => self.editor.undo(),
            "edit.redo" => self.editor.redo(),
            "edit.select_all" => self.editor.select_all(),
            "edit.find" | "find" => self.show_find_bar = true,
            "edit.replace" | "replace" => self.show_find_bar = true,
            "format.bold" => self.editor.apply_character_format(&CharacterFormatOp::ToggleBold),
            "format.italic" => self.editor.apply_character_format(&CharacterFormatOp::ToggleItalic),
            "format.underline" => self.editor.apply_character_format(&CharacterFormatOp::ToggleUnderline),
            "format.clear" => self.editor.clear_formatting(),
            "view.toggle_ruler" => {
                self.render_config.show_rulers = !self.render_config.show_rulers;
                self.config.show_ruler = self.render_config.show_rulers;
            }
            "view.toggle_formatting_marks" => {
                self.render_config.show_formatting_marks = !self.render_config.show_formatting_marks;
                self.config.show_formatting_marks = self.render_config.show_formatting_marks;
            }
            "view.toggle_navigation" => self.show_nav_pane = !self.show_nav_pane,
            "view.zoom_100" => {
                self.status_bar.zoom_percent = 100;
                self.render_config.zoom = 1.0;
            }
            "review.track_changes" => {
                self.tracking_config.enabled = !self.tracking_config.enabled;
                self.editor.track_changes = self.tracking_config.enabled;
                self.update_status_bar();
            }
            other => log::debug!("Unhandled ribbon action: {}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// Insert helpers
// ---------------------------------------------------------------------------

impl RustWriter {
    fn insert_table(&mut self, rows: usize, cols: usize) {
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
        let section = self.editor.cursor.position.section;
        let block = self.editor.cursor.position.block;
        if section < self.editor.document.sections.len() {
            self.editor.document.sections[section]
                .content
                .insert(block + 1, Block::Table(table));
        }
        self.update_status_bar();
    }

    fn insert_image_at_cursor(&mut self, path: &str) {
        use rw_document::inline::{ImageSource, InlineImage};
        use rw_document::{ElementId, Inline, Twips};

        let img = InlineImage {
            id: ElementId::new(),
            source: ImageSource::File(path.to_string()),
            width: Twips::from_inches(3.0),
            height: Twips::from_inches(2.0),
            alt_text: None,
            title: Some(path.to_string()),
        };
        let sec = self.editor.cursor.position.section;
        let blk = self.editor.cursor.position.block;
        if let Some(section) = self.editor.document.sections.get_mut(sec) {
            if let Some(rw_document::Block::Paragraph(para)) = section.content.get_mut(blk) {
                para.content.push(Inline::Image(img));
            }
        }
        self.update_status_bar();
    }

    fn insert_hyperlink_at_cursor(&mut self, url: &str, display_text: &str) {
        use rw_document::inline::{Hyperlink, HyperlinkTarget};
        use rw_document::{ElementId, Inline, TextRun};

        let link = Hyperlink {
            id: ElementId::new(),
            target: HyperlinkTarget::Url(url.to_string()),
            tooltip: None,
            content: vec![Inline::Text(TextRun::new(display_text))],
        };
        let sec = self.editor.cursor.position.section;
        let blk = self.editor.cursor.position.block;
        if let Some(section) = self.editor.document.sections.get_mut(sec) {
            if let Some(rw_document::Block::Paragraph(para)) = section.content.get_mut(blk) {
                para.content.push(Inline::Hyperlink(link));
            }
        }
        self.update_status_bar();
    }

    fn insert_bookmark_at_cursor(&mut self, name: &str) {
        use rw_document::inline::BookmarkMark;
        use rw_document::{ElementId, Inline};

        let id = ElementId::new();
        let sec = self.editor.cursor.position.section;
        let blk = self.editor.cursor.position.block;
        if let Some(section) = self.editor.document.sections.get_mut(sec) {
            if let Some(rw_document::Block::Paragraph(para)) = section.content.get_mut(blk) {
                para.content.push(Inline::BookmarkStart(BookmarkMark {
                    id,
                    name: name.to_string(),
                }));
                para.content.push(Inline::BookmarkEnd(BookmarkMark {
                    id,
                    name: name.to_string(),
                }));
            }
        }
    }

    fn insert_field_at_cursor(&mut self, field_type: rw_document::inline::FieldType) {
        use rw_document::inline::FieldRef;
        use rw_document::properties::CharacterProperties;
        use rw_document::{ElementId, Inline};

        let field = FieldRef {
            id: ElementId::new(),
            field_type,
            cached_value: None,
            properties: CharacterProperties::default(),
        };
        let sec = self.editor.cursor.position.section;
        let blk = self.editor.cursor.position.block;
        if let Some(section) = self.editor.document.sections.get_mut(sec) {
            if let Some(rw_document::Block::Paragraph(para)) = section.content.get_mut(blk) {
                para.content.push(Inline::Field(field));
            }
        }
        self.update_status_bar();
    }
}

// ---------------------------------------------------------------------------
// View builders
// ---------------------------------------------------------------------------

impl RustWriter {
    fn build_document_preview(&self) -> Element<'_, Message> {
        let mut col = widget::column().spacing(2);
        let mut para_count = 0;

        for section in &self.editor.document.sections {
            for block in &section.content {
                match block {
                    rw_document::block::Block::Paragraph(para) => {
                        let text = para.plain_text();
                        let is_cursor_here = self.editor.cursor.position.block == para_count;
                        let display_text = if text.is_empty() {
                            if is_cursor_here { "|".to_string() } else { " ".to_string() }
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
                        col = col.push(widget::text::caption(
                            format!("[Table: {}x{}]", table.rows, table.cols),
                        ));
                        para_count += 1;
                    }
                    rw_document::block::Block::HorizontalRule(_) => {
                        col = col.push(widget::divider::horizontal::default());
                        para_count += 1;
                    }
                    _ => { para_count += 1; }
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

    fn build_backstage_view(&self) -> Element<'_, Message> {
        let nav_items = [
            ("Info", BackstagePage::Info),
            ("New", BackstagePage::New),
            ("Open", BackstagePage::Open),
            ("Recent", BackstagePage::Recent),
            ("Save", BackstagePage::Save),
            ("Save As", BackstagePage::SaveAs),
            ("Print", BackstagePage::Print),
            ("Export", BackstagePage::Export),
            ("Options", BackstagePage::Options),
        ];

        let mut nav_col = widget::column().spacing(4).width(Length::Fixed(200.0));
        nav_col = nav_col.push(
            widget::button::text("< Back")
                .on_press(Message::CloseBackstage),
        );
        nav_col = nav_col.push(widget::divider::horizontal::default());
        for (label, page) in &nav_items {
            let btn = if self.backstage_page == *page {
                widget::button::suggested(*label)
                    .on_press(Message::BackstageNavigate(*page))
            } else {
                widget::button::text(*label)
                    .on_press(Message::BackstageNavigate(*page))
            };
            nav_col = nav_col.push(btn);
        }

        // Content area depends on active backstage page
        let content = match self.backstage_page {
            BackstagePage::Info => {
                let title = self.document_title();
                let word_count = self.editor.document.word_count();
                let char_count = self.editor.document.plain_text().len();
                widget::column().spacing(8)
                    .push(widget::text::title3("Document Info"))
                    .push(widget::text::body(format!("Title: {}", title)))
                    .push(widget::text::body(format!("Words: {}", word_count)))
                    .push(widget::text::body(format!("Characters: {}", char_count)))
                    .push(widget::text::body(format!(
                        "Author: {}", self.editor.document.metadata.author.as_deref().unwrap_or("Unknown")
                    )))
                    .push(widget::text::body(format!(
                        "Modified: {}", self.editor.document.metadata.modified.format("%Y-%m-%d %H:%M")
                    )))
            }
            BackstagePage::Recent => {
                let mut col = widget::column().spacing(4)
                    .push(widget::text::title3("Recent Documents"));
                if self.config.recent_files.is_empty() {
                    col = col.push(widget::text::body("No recent files."));
                } else {
                    for rf in &self.config.recent_files {
                        col = col.push(
                            widget::button::text(rf.name.as_str())
                                .on_press(Message::OpenFile(rf.path.clone())),
                        );
                    }
                }
                col
            }
            BackstagePage::SaveAs => {
                widget::column().spacing(8)
                    .push(widget::text::title3("Save As"))
                    .push(widget::text::body("Select a format:"))
                    .push(widget::button::text("ODF (.odt)").on_press(Message::RibbonAction("save_as_odt".into())))
                    .push(widget::button::text("DOCX (.docx)").on_press(Message::RibbonAction("save_as_docx".into())))
                    .push(widget::button::text("RTF (.rtf)").on_press(Message::RibbonAction("save_as_rtf".into())))
                    .push(widget::button::text("HTML (.html)").on_press(Message::RibbonAction("save_as_html".into())))
                    .push(widget::button::text("Plain Text (.txt)").on_press(Message::RibbonAction("save_as_txt".into())))
            }
            BackstagePage::Export => {
                widget::column().spacing(8)
                    .push(widget::text::title3("Export"))
                    .push(widget::text::body("Export document as:"))
                    .push(widget::button::text("PDF").on_press(Message::RibbonAction("export_pdf".into())))
                    .push(widget::button::text("HTML").on_press(Message::RibbonAction("export_html".into())))
                    .push(widget::button::text("EPUB").on_press(Message::RibbonAction("export_epub".into())))
            }
            BackstagePage::Print => {
                widget::column().spacing(8)
                    .push(widget::text::title3("Print"))
                    .push(widget::button::text("Print Document").on_press(Message::Print))
                    .push(widget::button::text("Print Preview").on_press(Message::PrintPreview))
            }
            BackstagePage::Options => {
                widget::column().spacing(8)
                    .push(widget::text::title3("Options"))
                    .push(widget::text::body(format!("Language: {}", self.config.language)))
                    .push(widget::text::body(format!("Default format: {}", self.config.default_format)))
                    .push(widget::text::body(format!("Auto-save: {} sec", self.config.auto_save_interval)))
                    .push(widget::text::body(format!("Spell check: {}", self.config.spell_check)))
                    .push(widget::text::body(format!("Measurement: {}", self.config.measurement_unit)))
            }
            _ => {
                widget::column().spacing(8)
                    .push(widget::text::title3(format!("{:?}", self.backstage_page)))
            }
        };

        Element::from(
            widget::row().spacing(8)
                .push(nav_col)
                .push(widget::divider::vertical::default())
                .push(
                    widget::container(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(16),
                )
                .width(Length::Fill)
                .height(Length::Fill),
        )
    }

    fn build_find_bar(&self) -> Element<'_, Message> {
        let find_input = widget::text_input("Find...", &self.find_query)
            .on_input(|q| Message::FindNext(q));
        let count_label = match self.last_find_count {
            Some(n) => widget::text::caption(format!("{} found", n)),
            None => widget::text::caption(""),
        };
        let close_btn = widget::button::icon(widget::icon::from_name("window-close-symbolic"))
            .on_press(Message::CloseFindReplace);
        Element::from(
            widget::row().spacing(8).padding(4)
                .push(find_input)
                .push(count_label)
                .push(close_btn),
        )
    }

    fn build_command_palette(&self) -> Element<'_, Message> {
        let results = self.command_registry.search(&self.command_palette.query);
        let mut list_col = widget::column().spacing(2);
        for cmd in results.iter().take(15) {
            let label = if let Some(shortcut) = &cmd.shortcut {
                format!("{} ({})", cmd.label, shortcut)
            } else {
                cmd.label.clone()
            };
            list_col = list_col.push(
                widget::button::text(label)
                    .on_press(Message::PaletteItemSelected(cmd.id.clone())),
            );
        }

        Element::from(
            widget::container(
                widget::column().spacing(4).width(Length::Fixed(400.0))
                    .push(
                        widget::text_input("Type a command...", &self.command_palette.query)
                            .on_input(Message::PaletteQueryChanged),
                    )
                    .push(
                        widget::scrollable(list_col)
                            .height(Length::Fixed(300.0)),
                    ),
            )
            .padding(8)
            .width(Length::Fixed(420.0)),
        )
    }

    fn build_navigation_pane(&self) -> Element<'_, Message> {
        let mut col = widget::column().spacing(4).width(Length::Fixed(200.0));
        col = col.push(widget::text::title4("Navigation"));
        col = col.push(widget::divider::horizontal::default());

        // List headings from the document
        let mut heading_count = 0;
        for section in &self.editor.document.sections {
            for block in &section.content {
                if let rw_document::block::Block::Paragraph(para) = block {
                    if let Some(ref style) = para.properties.paragraph_style {
                        if style.starts_with("Heading") {
                            let text = para.plain_text();
                            if !text.is_empty() {
                                col = col.push(widget::text::body(text));
                                heading_count += 1;
                            }
                        }
                    }
                }
            }
        }
        if heading_count == 0 {
            col = col.push(widget::text::caption("No headings found."));
        }

        Element::from(col.padding(8))
    }
}
