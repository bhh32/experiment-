use serde::{Deserialize, Serialize};

/// The root document — analogous to w:document/w:body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub children: Vec<Block>,
}

/// Block-level elements — analogous to w:p, w:tbl, w:sectPr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Paragraph),
    Heading(Heading),
    List(List),
    Table(Table),
    CodeBlock(CodeBlock),
    BlockQuote(Vec<Block>),
    ThematicBreak,
    PageBreak,
}

/// A paragraph — analogous to w:p with w:pPr + w:r children.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub runs: Vec<Run>,
    pub properties: ParaProperties,
}

/// Paragraph properties — analogous to w:pPr.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParaProperties {
    pub alignment: Alignment,
    /// Spacing before in points
    pub space_before_pt: f32,
    /// Spacing after in points
    pub space_after_pt: f32,
}

/// A heading — separate from Paragraph for semantic clarity.
/// Maps to w:p with w:pStyle = "Heading1" etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8, // 1-6
    pub runs: Vec<Run>,
    pub properties: ParaProperties,
}

/// A text run — analogous to w:r with w:rPr + w:t.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub text: String,
    pub properties: RunProperties,
}

/// Run properties — analogous to w:rPr.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunProperties {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link_url: Option<String>,
}

/// Text alignment — analogous to w:jc values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

impl Alignment {
    pub fn from_str(s: &str) -> Self {
        match s {
            "center" => Self::Center,
            "right" => Self::Right,
            "justify" => Self::Justify,
            _ => Self::Left,
        }
    }

    pub fn css_value(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Justify => "justify",
        }
    }
}

/// A list — analogous to a sequence of w:p with w:numPr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    /// Inline content of this list item
    pub runs: Vec<Run>,
    /// Nested blocks (sub-lists, paragraphs within a list item)
    pub children: Vec<Block>,
    /// Task list checkbox state: None = not a task, Some(true) = checked
    pub checked: Option<bool>,
}

/// A table — analogous to w:tbl.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub header: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub runs: Vec<Run>,
    pub alignment: Alignment,
    pub is_header: bool,
}

/// A fenced code block — maps to w:p with monospace font.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
}

/// A line break within a run (from hardbreaks).
impl Run {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            text: s.into(),
            properties: RunProperties::default(),
        }
    }

    pub fn line_break() -> Self {
        Self {
            text: "\n".to_string(),
            properties: RunProperties::default(),
        }
    }

    pub fn with_bold(mut self) -> Self {
        self.properties.bold = true;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.properties.italic = true;
        self
    }

    pub fn with_strikethrough(mut self) -> Self {
        self.properties.strikethrough = true;
        self
    }

    pub fn with_code(mut self) -> Self {
        self.properties.code = true;
        self
    }

    pub fn with_link(mut self, url: impl Into<String>) -> Self {
        self.properties.link_url = Some(url.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_roundtrip() {
        assert_eq!(Alignment::from_str("center"), Alignment::Center);
        assert_eq!(Alignment::from_str("right"), Alignment::Right);
        assert_eq!(Alignment::from_str("justify"), Alignment::Justify);
        assert_eq!(Alignment::from_str("left"), Alignment::Left);
        assert_eq!(Alignment::from_str("unknown"), Alignment::Left);
    }

    #[test]
    fn run_builder() {
        let run = Run::text("hello").with_bold().with_italic();
        assert!(run.properties.bold);
        assert!(run.properties.italic);
        assert_eq!(run.text, "hello");
    }

    #[test]
    fn default_paragraph() {
        let p = Paragraph {
            runs: vec![Run::text("test")],
            properties: ParaProperties::default(),
        };
        assert_eq!(p.properties.alignment, Alignment::Left);
    }
}
