//! Drawing shapes — rectangles, ovals, lines, arrows, callouts, etc.

use serde::{Deserialize, Serialize};
use rw_document::Twips;

/// A drawing shape in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub fill: Option<ShapeFill>,
    pub outline: Option<ShapeOutline>,
    pub text_content: Option<String>,
    /// Display width in twips
    pub width: Twips,
    /// Display height in twips
    pub height: Twips,
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

/// Builder for constructing shapes fluently.
#[derive(Debug)]
pub struct ShapeBuilder {
    shape_type: ShapeType,
    fill: Option<ShapeFill>,
    outline: Option<ShapeOutline>,
    text_content: Option<String>,
    width: Twips,
    height: Twips,
}

impl ShapeBuilder {
    /// Start building a shape of the given type.
    pub fn new(shape_type: ShapeType) -> Self {
        Self {
            shape_type,
            fill: None,
            outline: None,
            text_content: None,
            width: Twips::from_inches(1.0),
            height: Twips::from_inches(1.0),
        }
    }

    /// Set the fill for this shape.
    pub fn with_fill(mut self, fill: ShapeFill) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Set the outline for this shape.
    pub fn with_outline(mut self, outline: ShapeOutline) -> Self {
        self.outline = Some(outline);
        self
    }

    /// Set the display size.
    pub fn with_size(mut self, width: Twips, height: Twips) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set text content inside the shape.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_content = Some(text.into());
        self
    }

    /// Build the final `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            shape_type: self.shape_type,
            fill: self.fill,
            outline: self.outline,
            text_content: self.text_content,
            width: self.width,
            height: self.height,
        }
    }
}
