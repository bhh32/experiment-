//! Main application struct implementing cosmic::Application.

use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::{Application, Element};

use crate::messages::Message;
use rw_editor::EditorState;
use rw_layout::LayoutEngine;
use rw_render::RenderConfig;
use rw_ribbon::{RibbonDisplayMode, TabId};
use rw_widgets::status_bar::{StatusBarState, ViewMode};
use rw_command_palette::PaletteState;

/// The main Rust Writer application.
pub struct RustWriter {
    core: Core,
    /// The editor state (document + cursor + undo)
    editor: EditorState,
    /// The layout engine
    layout_engine: LayoutEngine,
    /// Render configuration
    render_config: RenderConfig,
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
        let mut status_bar = StatusBarState::default();
        status_bar.page = 1;
        status_bar.total_pages = 1;
        status_bar.language = "English (US)".to_string();
        status_bar.zoom_percent = 100;

        let app = Self {
            core,
            editor: EditorState::new(),
            layout_engine: LayoutEngine::new(),
            render_config: RenderConfig::default(),
            active_tab: TabId::Home,
            ribbon_mode: RibbonDisplayMode::Full,
            status_bar,
            command_palette: PaletteState::default(),
            file_path: None,
            focus_mode: false,
            view_mode: ViewMode::PrintLayout,
        };

        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    fn view(&self) -> Element<Self::Message> {
        // Main application layout:
        // ┌─────────────────────────────────────────┐
        // │  Quick Access Toolbar                     │
        // ├─────────────────────────────────────────┤
        // │  Ribbon (tabbed toolbar)                  │
        // ├──────┬──────────────────────────┬───────┤
        // │      │                          │       │
        // │ Rule │   Document Canvas        │ Side  │
        // │  r   │   (pages with content)   │ bar   │
        // │      │                          │       │
        // ├──────┴──────────────────────────┴───────┤
        // │  Status Bar                              │
        // └─────────────────────────────────────────┘

        let title = if let Some(path) = &self.file_path {
            std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        } else {
            "Untitled".to_string()
        };

        let dirty_indicator = if self.editor.is_dirty() { " *" } else { "" };

        let content = widget::column()
            .push(
                widget::text::title4(format!(
                    "Rust Writer — {}{}",
                    title, dirty_indicator
                ))
            )
            .push(
                widget::text::body(format!(
                    "Page {} of {} | {} words | Zoom: {}%",
                    self.status_bar.page,
                    self.status_bar.total_pages,
                    self.editor.document.word_count(),
                    self.status_bar.zoom_percent,
                ))
            )
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        Element::from(content)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::None => {}
            Message::NewDocument => {
                self.editor = EditorState::new();
                self.file_path = None;
            }
            Message::TabChanged(tab) => {
                self.active_tab = tab;
            }
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
        }
        Task::none()
    }
}
