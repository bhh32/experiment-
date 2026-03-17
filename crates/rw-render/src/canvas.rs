//! Canvas-based document rendering.
//!
//! Renders the laid-out document pages onto an iced Canvas widget.
//! Handles drawing of text, images, shapes, selection highlights,
//! cursors, page boundaries, and decorations.

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
