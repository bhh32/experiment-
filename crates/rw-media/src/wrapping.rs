//! Text wrapping calculations for floating objects.

/// Text wrapping configuration for a floating object.
#[derive(Debug, Clone, Copy)]
pub struct WrapConfig {
    pub mode: WrapMode,
    pub side: WrapSide,
    pub distance_top: i32,    // twips
    pub distance_bottom: i32,
    pub distance_left: i32,
    pub distance_right: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    /// No wrapping (text behind or in front)
    None,
    /// Square wrapping around bounding box
    Square,
    /// Tight wrapping around shape outline
    Tight,
    /// Text flows through transparent areas
    Through,
    /// Text above and below only
    TopAndBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapSide {
    Both,
    Left,
    Right,
    Largest,
}

impl Default for WrapConfig {
    fn default() -> Self {
        Self {
            mode: WrapMode::Square,
            side: WrapSide::Both,
            distance_top: 0,
            distance_bottom: 0,
            distance_left: 72,  // ~1mm
            distance_right: 72,
        }
    }
}

/// A rectangular bounding region in layout coordinates (twips).
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn left(&self) -> i32 { self.x }
    pub fn right(&self) -> i32 { self.x + self.width }
    pub fn top(&self) -> i32 { self.y }
    pub fn bottom(&self) -> i32 { self.y + self.height }
}

/// A horizontal segment of available text width on a single line.
#[derive(Debug, Clone, Copy)]
pub struct WrapSegment {
    /// Y coordinate of the line (top edge, twips)
    pub y: i32,
    /// Line height (twips)
    pub height: i32,
    /// Available left margin for text (twips from page left)
    pub text_left: i32,
    /// Available right margin for text (twips from page left)
    pub text_right: i32,
}

/// Calculate wrap segments — the available horizontal extents for text on
/// each line that overlaps with the shape/image bounds.
///
/// Returns one `WrapSegment` per line-height band within the vertical extent
/// of the shape bounds.
///
/// `shape_bounds` — bounding box of the floating object (including wrap distance).
/// `text_bounds` — the full text column bounds (x=left margin, width=column width).
/// `wrap_mode`   — how the text should wrap.
/// `line_height` — height of a single text line in twips (used to slice into bands).
pub fn calculate_wrap_points(
    shape_bounds: Rect,
    text_bounds: Rect,
    wrap_mode: WrapMode,
    line_height: i32,
) -> Vec<WrapSegment> {
    if line_height <= 0 || wrap_mode == WrapMode::None {
        return Vec::new();
    }

    let mut segments = Vec::new();

    match wrap_mode {
        WrapMode::None => {}

        WrapMode::TopAndBottom => {
            // Text only above and below the shape — no segments within the shape
            // (the caller is responsible for skipping lines inside the shape).
        }

        WrapMode::Square | WrapMode::Tight | WrapMode::Through => {
            // Divide the shape's vertical extent into line-height bands
            let start_y = shape_bounds.top();
            let end_y = shape_bounds.bottom();

            let mut y = start_y;
            while y < end_y {
                // Left segment: from text column left to shape left
                let left_available_width = shape_bounds.left() - text_bounds.left();
                // Right segment: from shape right to text column right
                let right_available_width = text_bounds.right() - shape_bounds.right();

                // Choose which side(s) to wrap based on available space
                // For simplicity we produce the widest available region
                let (text_left, text_right) = if left_available_width >= right_available_width {
                    (text_bounds.left(), shape_bounds.left())
                } else {
                    (shape_bounds.right(), text_bounds.right())
                };

                if text_right > text_left {
                    segments.push(WrapSegment {
                        y,
                        height: line_height,
                        text_left,
                        text_right,
                    });
                }

                y += line_height;
            }
        }
    }

    segments
}
