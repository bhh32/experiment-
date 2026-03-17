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
