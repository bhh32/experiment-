//! Drawing shapes — rectangles, ovals, lines, arrows, callouts, etc.

use serde::{Deserialize, Serialize};

/// A drawing shape in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub fill: Option<ShapeFill>,
    pub outline: Option<ShapeOutline>,
    pub text_content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    RoundedRectangle,
    Oval,
    Triangle,
    Diamond,
    Pentagon,
    Hexagon,
    Line,
    Arrow,
    DoubleArrow,
    CurvedArrow,
    Callout,
    CloudCallout,
    Star4,
    Star5,
    Star6,
    Heart,
    Lightning,
    FlowchartProcess,
    FlowchartDecision,
    FlowchartTerminator,
    FlowchartData,
    Freeform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeFill {
    Solid(String),        // hex color
    Gradient(Vec<GradientStop>),
    Pattern(String),      // pattern name
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: f64,    // 0.0 - 1.0
    pub color: String,    // hex color
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeOutline {
    pub color: String,
    pub width_twips: i32,
    pub dash_style: DashStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashStyle {
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
}
