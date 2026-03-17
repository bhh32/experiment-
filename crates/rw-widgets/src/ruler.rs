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
