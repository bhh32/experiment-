use serde::{Deserialize, Serialize};

/// Schema version — increment when adding/removing fields.
pub const SCHEMA_VERSION: u32 = 2;

/// The root document — analogous to w:document/w:body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub schema_version: u32,
    pub children: Vec<Block>,
    pub header: Option<HeaderFooter>,
    pub footer: Option<HeaderFooter>,
    pub footnotes: Vec<Footnote>,
    pub comments: Vec<Comment>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            children: Vec::new(),
            header: None,
            footer: None,
            footnotes: Vec::new(),
            comments: Vec::new(),
        }
    }
}

/// Header or footer content — analogous to w:hdr / w:ftr.
/// Supports `{pagenumber}` placeholder in text for automatic page numbering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooter {
    pub runs: Vec<Run>,
    pub alignment: Alignment,
}

/// A footnote — analogous to w:footnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    /// Unique ID referenced from inline footnote markers
    pub id: u32,
    /// Content of the footnote
    pub runs: Vec<Run>,
}

/// A comment — analogous to w:comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u32,
    pub author: String,
    pub date: String,
    pub runs: Vec<Run>,
    /// The text range this comment applies to (start marker in content)
    pub anchor_text: String,
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
    Image(Image),
    ThematicBreak,
    PageBreak,
    /// A section break — different from page break in that it can change
    /// page layout (margins, orientation) for subsequent content.
    SectionBreak,
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
    pub space_before_pt: f32,
    pub space_after_pt: f32,
    /// Indent left in points
    pub indent_left_pt: f32,
    /// Indent right in points
    pub indent_right_pt: f32,
    /// First line indent (positive) or hanging indent (negative) in points
    pub indent_first_line_pt: f32,
    /// Named paragraph style (e.g. "Heading1", "BodyText", "Caption")
    pub style: Option<String>,
}

/// A heading — separate from Paragraph for semantic clarity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
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
    pub underline: bool,
    pub strikethrough: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub code: bool,
    /// Hex color (e.g. "FF0000" for red), None = default/auto
    pub color: Option<String>,
    /// Highlight color name (e.g. "yellow", "green")
    pub highlight: Option<String>,
    /// Font family override for this run
    pub font: Option<String>,
    /// Font size override in points for this run
    pub size_pt: Option<f32>,
    /// Hyperlink URL
    pub link_url: Option<String>,
    /// Footnote reference ID — this run is a footnote marker
    pub footnote_ref: Option<u32>,
    /// Comment reference ID — this run marks the start of a commented range
    pub comment_ref: Option<u32>,
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
    pub runs: Vec<Run>,
    pub children: Vec<Block>,
    pub checked: Option<bool>,
}

/// A table — analogous to w:tbl with w:tblPr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub header: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
    pub properties: TableProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableProperties {
    pub width_pct: u32,
    pub column_alignments: Vec<Alignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub runs: Vec<Run>,
    /// Nested block content (paragraphs, lists inside a cell)
    pub blocks: Vec<Block>,
    pub alignment: Alignment,
    pub is_header: bool,
    pub col_span: u32,
    pub row_span: u32,
    /// Background color as hex (e.g. "E8E8E8")
    pub shading: Option<String>,
}

/// A fenced code block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
}

/// An image — analogous to w:drawing/wp:inline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    pub alt: String,
    pub title: String,
    pub width_px: u32,
    pub height_px: u32,
}

// ─── Builders ───

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

    pub fn with_underline(mut self) -> Self {
        self.properties.underline = true;
        self
    }

    pub fn with_strikethrough(mut self) -> Self {
        self.properties.strikethrough = true;
        self
    }

    pub fn with_superscript(mut self) -> Self {
        self.properties.superscript = true;
        self
    }

    pub fn with_subscript(mut self) -> Self {
        self.properties.subscript = true;
        self
    }

    pub fn with_code(mut self) -> Self {
        self.properties.code = true;
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.properties.color = Some(color.into());
        self
    }

    pub fn with_highlight(mut self, color: impl Into<String>) -> Self {
        self.properties.highlight = Some(color.into());
        self
    }

    pub fn with_font(mut self, font: impl Into<String>) -> Self {
        self.properties.font = Some(font.into());
        self
    }

    pub fn with_size(mut self, pt: f32) -> Self {
        self.properties.size_pt = Some(pt);
        self
    }

    pub fn with_link(mut self, url: impl Into<String>) -> Self {
        self.properties.link_url = Some(url.into());
        self
    }

    pub fn with_footnote_ref(mut self, id: u32) -> Self {
        self.properties.footnote_ref = Some(id);
        self
    }
}

impl TableCell {
    pub fn new(runs: Vec<Run>, is_header: bool) -> Self {
        Self {
            runs,
            blocks: Vec::new(),
            alignment: Alignment::Left,
            is_header,
            col_span: 1,
            row_span: 1,
            shading: None,
        }
    }

    pub fn with_shading(mut self, color: impl Into<String>) -> Self {
        self.shading = Some(color.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version() {
        let doc = Document::new();
        assert_eq!(doc.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn alignment_roundtrip() {
        assert_eq!(Alignment::from_str("center"), Alignment::Center);
        assert_eq!(Alignment::from_str("right"), Alignment::Right);
        assert_eq!(Alignment::from_str("justify"), Alignment::Justify);
        assert_eq!(Alignment::from_str("left"), Alignment::Left);
        assert_eq!(Alignment::from_str("unknown"), Alignment::Left);
    }

    #[test]
    fn run_builder_all_props() {
        let run = Run::text("hello")
            .with_bold()
            .with_italic()
            .with_underline()
            .with_color("FF0000")
            .with_highlight("yellow")
            .with_superscript()
            .with_font("Arial")
            .with_size(14.0);
        assert!(run.properties.bold);
        assert!(run.properties.italic);
        assert!(run.properties.underline);
        assert!(run.properties.superscript);
        assert_eq!(run.properties.color.as_deref(), Some("FF0000"));
        assert_eq!(run.properties.highlight.as_deref(), Some("yellow"));
        assert_eq!(run.properties.font.as_deref(), Some("Arial"));
        assert_eq!(run.properties.size_pt, Some(14.0));
    }

    #[test]
    fn table_cell_builder() {
        let cell = TableCell::new(vec![Run::text("data")], false)
            .with_shading("E8E8E8");
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
        assert_eq!(cell.shading.as_deref(), Some("E8E8E8"));
    }

    #[test]
    fn footnote_ref() {
        let run = Run::text("1").with_footnote_ref(1);
        assert_eq!(run.properties.footnote_ref, Some(1));
    }
}
