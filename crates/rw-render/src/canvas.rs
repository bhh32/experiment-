//! Canvas-based document rendering.
//!
//! Renders the laid-out document pages onto an iced Canvas widget.
//! Handles drawing of text, images, shapes, selection highlights,
//! cursors, page boundaries, and decorations.

use crate::theme::RenderTheme;
use crate::viewport::Viewport;
use crate::RenderConfig;
use rw_layout::line::LayoutLine;
use rw_layout::page::LayoutPage;
use rw_layout::LayoutResult;

/// Drawing commands for the document renderer.
/// These are abstract commands that get translated to iced Canvas operations.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    FillRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: String,
    },
    /// Draw a stroked rectangle
    StrokeRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: String,
        line_width: f64,
    },
    /// Draw a line
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: String,
        line_width: f64,
    },
    /// Draw text at a position
    Text {
        text: String,
        x: f64,
        y: f64,
        font_family: String,
        font_size: f64,
        bold: bool,
        italic: bool,
        color: String,
    },
    /// Draw an image
    Image {
        resource_id: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    /// Apply a clipping rectangle
    Clip {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    /// Save the current transform state
    Save,
    /// Restore a previously saved transform state
    Restore,
    /// Translate the coordinate system
    Translate { x: f64, y: f64 },
    /// Scale the coordinate system
    Scale { x: f64, y: f64 },
}

// ---------------------------------------------------------------------------
// DocumentRenderer
// ---------------------------------------------------------------------------

/// Converts a `LayoutResult` into a flat list of `DrawCommand`s that can
/// be fed to any canvas/GPU back-end.
pub struct DocumentRenderer {
    config: RenderConfig,
    theme: RenderTheme,
    /// Vertical gap between pages in document points
    page_gap: f64,
}

impl DocumentRenderer {
    /// Create a new renderer with the given config and theme.
    pub fn new(config: &RenderConfig, theme: &RenderTheme) -> Self {
        Self {
            config: config.clone(),
            theme: theme.clone(),
            page_gap: config.page_gap,
        }
    }

    /// Generate draw commands for all pages visible in the viewport.
    ///
    /// The coordinate system used for `DrawCommand` values is **screen pixels**
    /// (origin = top-left of the viewport, Y increases downward).
    pub fn render_document(
        &self,
        layout: &LayoutResult,
        viewport: &Viewport,
    ) -> Vec<DrawCommand> {
        let mut cmds: Vec<DrawCommand> = Vec::new();

        // Fill the desk (background behind pages)
        cmds.push(DrawCommand::FillRect {
            x: 0.0,
            y: 0.0,
            width: viewport.width,
            height: viewport.height,
            color: self.theme.desk_color.clone(),
        });

        // Lay pages out vertically with page_gap between them
        let mut doc_y = self.page_gap;

        for page in &layout.pages {
            let page_h = page.bounds.height * viewport.zoom;
            let page_w = page.bounds.width * viewport.zoom;

            // Centre the page horizontally in the viewport
            let page_x = (viewport.width - page_w) / 2.0;
            let (screen_x, screen_y) = viewport.doc_to_screen(page_x / viewport.zoom, doc_y);

            // Visibility culling
            if viewport.is_visible(
                page_x / viewport.zoom,
                doc_y,
                page.bounds.width,
                page.bounds.height,
            ) {
                let mut page_cmds = self.render_page(page, screen_y);
                // Translate all commands so the page starts at (screen_x, screen_y)
                cmds.push(DrawCommand::Save);
                cmds.push(DrawCommand::Translate {
                    x: screen_x,
                    y: screen_y,
                });
                cmds.append(&mut page_cmds);
                cmds.push(DrawCommand::Restore);
            }

            doc_y += page.bounds.height + self.page_gap;
            let _ = page_h; // suppress unused warning
        }

        cmds
    }

    /// Render a single page.
    ///
    /// Returns commands with origin at the top-left of the page.
    pub fn render_page(&self, page: &LayoutPage, _offset_y: f64) -> Vec<DrawCommand> {
        let mut cmds: Vec<DrawCommand> = Vec::new();
        let z = self.config.zoom;
        let pw = page.bounds.width * z;
        let ph = page.bounds.height * z;

        // Page shadow
        if self.config.page_shadow {
            let shadow_offset = 4.0;
            cmds.push(DrawCommand::FillRect {
                x: shadow_offset,
                y: shadow_offset,
                width: pw,
                height: ph,
                color: self.theme.page_shadow_color.clone(),
            });
        }

        // Page background
        cmds.push(DrawCommand::FillRect {
            x: 0.0,
            y: 0.0,
            width: pw,
            height: ph,
            color: self.theme.page_color.clone(),
        });

        // Page border
        if self.config.show_page_boundaries {
            cmds.push(DrawCommand::StrokeRect {
                x: 0.0,
                y: 0.0,
                width: pw,
                height: ph,
                color: "#CCCCCC".to_string(),
                line_width: 1.0,
            });
        }

        // Clip to page area
        cmds.push(DrawCommand::Clip {
            x: 0.0,
            y: 0.0,
            width: pw,
            height: ph,
        });

        // Render content lines
        for line in &page.lines {
            let mut line_cmds = self.render_line(line);
            cmds.append(&mut line_cmds);
        }

        // Margin guides
        if self.config.show_page_boundaries {
            let ma = &page.properties.margins;
            let ml = ma.left.to_points() * z;
            let mt = ma.top.to_points() * z;
            let mr = pw - ma.right.to_points() * z;
            let mb = ph - ma.bottom.to_points() * z;
            let guide = self.theme.margin_guide_color.clone();
            // Left margin
            cmds.push(DrawCommand::Line { x1: ml, y1: 0.0, x2: ml, y2: ph, color: guide.clone(), line_width: 0.5 });
            // Right margin
            cmds.push(DrawCommand::Line { x1: mr, y1: 0.0, x2: mr, y2: ph, color: guide.clone(), line_width: 0.5 });
            // Top margin
            cmds.push(DrawCommand::Line { x1: 0.0, y1: mt, x2: pw, y2: mt, color: guide.clone(), line_width: 0.5 });
            // Bottom margin
            cmds.push(DrawCommand::Line { x1: 0.0, y1: mb, x2: pw, y2: mb, color: guide.clone(), line_width: 0.5 });
        }

        cmds
    }

    /// Render a single line of text.
    ///
    /// Returns commands with coordinates already in page-local points
    /// scaled by the zoom factor.
    pub fn render_line(&self, line: &LayoutLine) -> Vec<DrawCommand> {
        let mut cmds: Vec<DrawCommand> = Vec::new();
        let z = self.config.zoom;

        if self.config.show_text_boundaries {
            cmds.push(DrawCommand::StrokeRect {
                x: line.bounds.x * z,
                y: line.bounds.y * z,
                width: line.bounds.width * z,
                height: line.bounds.height * z,
                color: "#FF000040".to_string(),
                line_width: 0.5,
            });
        }

        for run in &line.runs {
            if run.text.trim().is_empty() {
                continue;
            }
            cmds.push(DrawCommand::Text {
                text: run.text.clone(),
                x: run.bounds.x * z,
                y: (run.bounds.y + line.baseline) * z,
                font_family: run.font_family.clone(),
                font_size: run.font_size * z,
                bold: run.bold,
                italic: run.italic,
                color: run.color.clone(),
            });
        }

        cmds
    }

    /// Render a blinking cursor bar.
    ///
    /// `position` is in page-local points; the result is in screen coordinates
    /// (assuming the caller has already applied the page translation).
    pub fn render_cursor(&self, position: (f64, f64), height: f64) -> Vec<DrawCommand> {
        let z = self.config.zoom;
        vec![DrawCommand::FillRect {
            x: position.0 * z,
            y: position.1 * z,
            width: 2.0,
            height: height * z,
            color: self.theme.cursor_color.clone(),
        }]
    }

    /// Render a selection highlight between two page-local points.
    ///
    /// This is a simplified single-rectangle highlight; the caller should
    /// invoke this once per selected line for multi-line selections.
    pub fn render_selection(
        &self,
        start: (f64, f64),
        end: (f64, f64),
        height: f64,
    ) -> Vec<DrawCommand> {
        let z = self.config.zoom;
        let x = start.0.min(end.0) * z;
        let y = start.1.min(end.1) * z;
        let w = (end.0 - start.0).abs() * z;
        let h = height * z;

        vec![DrawCommand::FillRect {
            x,
            y,
            width: w,
            height: h,
            color: self.theme.selection_color.clone(),
        }]
    }
}
