//! Document ruler widget.
//!
//! Displays horizontal and vertical rulers alongside the document canvas.
//! Shows page margins, paragraph indents, and tab stops. Supports
//! interactive dragging to adjust margins, indents, and tab stops.

/// Ruler configuration.
#[derive(Debug, Clone)]
pub struct RulerConfig {
    /// Measurement unit
    pub unit: RulerUnit,
    /// Whether to show the ruler
    pub visible: bool,
    /// Major tick interval (in the chosen unit)
    pub major_tick: f64,
    /// Number of minor ticks between major ticks
    pub minor_ticks: u32,
}

impl Default for RulerConfig {
    fn default() -> Self {
        Self {
            unit: RulerUnit::Inches,
            visible: true,
            major_tick: 1.0,
            minor_ticks: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerUnit {
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
}

impl RulerUnit {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::Inches => "in",
            Self::Centimeters => "cm",
            Self::Millimeters => "mm",
            Self::Points => "pt",
            Self::Picas => "pc",
        }
    }

    /// Convert from twips to this unit.
    pub fn from_twips(&self, twips: i32) -> f64 {
        match self {
            Self::Inches => twips as f64 / 1440.0,
            Self::Centimeters => twips as f64 / 567.0,
            Self::Millimeters => twips as f64 / 56.7,
            Self::Points => twips as f64 / 20.0,
            Self::Picas => twips as f64 / 240.0,
        }
    }

    /// Convert from this unit to twips.
    pub fn to_twips(&self, value: f64) -> i32 {
        let twips = match self {
            Self::Inches => value * 1440.0,
            Self::Centimeters => value * 567.0,
            Self::Millimeters => value * 56.7,
            Self::Points => value * 20.0,
            Self::Picas => value * 240.0,
        };
        twips as i32
    }
}

/// Items that can be dragged on the ruler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerDragTarget {
    LeftMargin,
    RightMargin,
    FirstLineIndent,
    HangingIndent,
    LeftIndent,
    RightIndent,
    TabStop(usize),
}

/// A tab stop displayed on the ruler.
#[derive(Debug, Clone, Copy)]
pub struct RulerTabStop {
    /// Position from the left margin in twips
    pub position_twips: i32,
    /// Tab alignment type
    pub tab_type: RulerTabType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerTabType {
    Left,
    Center,
    Right,
    Decimal,
}

/// Full state of the ruler widget.
#[derive(Debug, Clone)]
pub struct RulerState {
    /// Measurement unit for display
    pub unit: RulerUnit,
    /// Left margin position in twips (from page edge)
    pub margin_left: i32,
    /// Right margin position in twips (from page edge)
    pub margin_right: i32,
    /// Left paragraph indent in twips (from left margin)
    pub indent_left: i32,
    /// Right paragraph indent in twips (from right margin)
    pub indent_right: i32,
    /// First-line indent (positive) or hanging indent (negative) in twips
    pub first_line_indent: i32,
    /// Tab stops for the current paragraph
    pub tab_stops: Vec<RulerTabStop>,
    /// Whether the user is currently dragging a target
    pub drag_target: Option<RulerDragTarget>,
    /// Page width in twips
    pub page_width: i32,
}

impl Default for RulerState {
    fn default() -> Self {
        Self {
            unit: RulerUnit::Inches,
            margin_left: 1440,  // 1"
            margin_right: 1440, // 1"
            indent_left: 0,
            indent_right: 0,
            first_line_indent: 0,
            tab_stops: Vec::new(),
            drag_target: None,
            page_width: 12240, // 8.5"
        }
    }
}

impl RulerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tab stop at the given position.
    pub fn add_tab_stop(&mut self, position_twips: i32, tab_type: RulerTabType) {
        self.tab_stops.push(RulerTabStop { position_twips, tab_type });
        self.tab_stops.sort_by_key(|t| t.position_twips);
    }

    /// Remove a tab stop by index.
    pub fn remove_tab_stop(&mut self, index: usize) {
        if index < self.tab_stops.len() {
            self.tab_stops.remove(index);
        }
    }

    /// Content width (page width minus both margins) in twips.
    pub fn content_width_twips(&self) -> i32 {
        self.page_width - self.margin_left - self.margin_right
    }
}
