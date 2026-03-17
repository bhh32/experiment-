//! Document canvas widget — the main editing area.
//!
//! This is the central widget that displays the document pages,
//! handles scroll/zoom, and routes mouse/keyboard events to the editor.

/// Scroll direction for wheel events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// The document canvas state.
#[derive(Debug, Clone)]
pub struct DocumentCanvasState {
    /// Horizontal scroll offset in document points
    pub scroll_x: f64,
    /// Vertical scroll offset in document points
    pub scroll_y: f64,
    /// Viewport width in screen pixels
    pub viewport_width: f64,
    /// Viewport height in screen pixels
    pub viewport_height: f64,
    /// Current zoom factor (1.0 = 100%)
    pub zoom: f64,
    /// Whether the cursor is currently visible (blink state)
    pub cursor_visible: bool,
    /// Accumulated blink timer ticks (reset each blink period)
    pub cursor_blink_timer: u32,
    /// Blink period in ticks (timer fires at ~30 Hz so 15 = 0.5 s)
    pub cursor_blink_period: u32,
    /// Whether there is an active text selection
    pub has_selection: bool,
    /// Mouse drag state
    pub drag: DragState,
}

impl Default for DocumentCanvasState {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
            zoom: 1.0,
            cursor_visible: true,
            cursor_blink_timer: 0,
            cursor_blink_period: 15,
            has_selection: false,
            drag: DragState::None,
        }
    }
}

impl DocumentCanvasState {
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            viewport_width,
            viewport_height,
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // Scrolling
    // -----------------------------------------------------------------------

    /// Scroll vertically by `delta` document points (positive = down).
    pub fn scroll_by(&mut self, delta_x: f64, delta_y: f64) {
        self.scroll_x = (self.scroll_x + delta_x).max(0.0);
        self.scroll_y = (self.scroll_y + delta_y).max(0.0);
    }

    /// Scroll to a specific document point position.
    pub fn scroll_to(&mut self, x: f64, y: f64) {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
    }

    /// Handle a mouse-wheel scroll event.
    pub fn handle_scroll(&mut self, direction: ScrollDirection, amount: f64) {
        let step = amount * 20.0; // 20 pts per scroll unit
        match direction {
            ScrollDirection::Up => self.scroll_by(0.0, -step),
            ScrollDirection::Down => self.scroll_by(0.0, step),
            ScrollDirection::Left => self.scroll_by(-step, 0.0),
            ScrollDirection::Right => self.scroll_by(step, 0.0),
        }
    }

    // -----------------------------------------------------------------------
    // Zoom
    // -----------------------------------------------------------------------

    /// Set the zoom level, clamped to [0.1, 5.0].
    pub fn set_zoom(&mut self, zoom: f64) {
        let old_zoom = self.zoom;
        self.zoom = zoom.clamp(0.1, 5.0);

        // Adjust scroll so the document centre stays centred
        if old_zoom != 0.0 {
            let ratio = self.zoom / old_zoom;
            self.scroll_x *= ratio;
            self.scroll_y *= ratio;
        }
    }

    // -----------------------------------------------------------------------
    // Cursor blink
    // -----------------------------------------------------------------------

    /// Advance the blink timer by one tick; flip visibility at period boundary.
    pub fn tick_cursor_blink(&mut self) {
        self.cursor_blink_timer += 1;
        if self.cursor_blink_timer >= self.cursor_blink_period {
            self.cursor_blink_timer = 0;
            self.cursor_visible = !self.cursor_visible;
        }
    }

    /// Reset the blink timer (e.g. after a keystroke, make cursor immediately visible).
    pub fn reset_cursor_blink(&mut self) {
        self.cursor_blink_timer = 0;
        self.cursor_visible = true;
    }

    // -----------------------------------------------------------------------
    // Coordinate conversion
    // -----------------------------------------------------------------------

    /// Convert a screen-pixel coordinate to a document-point coordinate.
    pub fn screen_to_doc(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        (
            (screen_x + self.scroll_x) / self.zoom,
            (screen_y + self.scroll_y) / self.zoom,
        )
    }

    /// Convert a document-point coordinate to a screen-pixel coordinate.
    pub fn doc_to_screen(&self, doc_x: f64, doc_y: f64) -> (f64, f64) {
        (
            doc_x * self.zoom - self.scroll_x,
            doc_y * self.zoom - self.scroll_y,
        )
    }
}

/// Mouse drag state for the canvas.
#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    /// No active drag
    None,
    /// Selecting text (click-and-drag)
    Selecting {
        start_doc_x: f64,
        start_doc_y: f64,
    },
    /// Scrolling via middle-click drag
    Panning {
        origin_scroll_x: f64,
        origin_scroll_y: f64,
        start_screen_x: f64,
        start_screen_y: f64,
    },
}
