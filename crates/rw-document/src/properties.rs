use crate::{Color, Twips};
use serde::{Deserialize, Serialize};

/// Character-level formatting properties.
/// All fields are `Option` to support style inheritance — `None` means
/// "inherit from the applied style or default."
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterProperties {
    /// Font family name (e.g., "Arial", "Times New Roman")
    pub font_family: Option<String>,
    /// Font size in half-points (e.g., 24 = 12pt)
    pub font_size: Option<u32>,
    /// Bold
    pub bold: Option<bool>,
    /// Italic
    pub italic: Option<bool>,
    /// Underline style
    pub underline: Option<UnderlineStyle>,
    /// Underline color (if different from text color)
    pub underline_color: Option<Color>,
    /// Strikethrough
    pub strikethrough: Option<StrikethroughStyle>,
    /// Text color
    pub color: Option<Color>,
    /// Highlight / background color
    pub highlight: Option<Color>,
    /// Superscript
    pub superscript: Option<bool>,
    /// Subscript
    pub subscript: Option<bool>,
    /// Small caps
    pub small_caps: Option<bool>,
    /// All caps
    pub all_caps: Option<bool>,
    /// Hidden text
    pub hidden: Option<bool>,
    /// Character spacing adjustment in twips
    pub spacing: Option<Twips>,
    /// Kerning threshold in half-points (0 = no kerning)
    pub kerning: Option<u32>,
    /// Vertical position adjustment in half-points (positive = raised)
    pub position: Option<i32>,
    /// Horizontal scale percentage (100 = normal)
    pub scale: Option<u32>,
    /// Text shadow
    pub shadow: Option<bool>,
    /// Outline effect
    pub outline: Option<bool>,
    /// Emboss effect
    pub emboss: Option<bool>,
    /// Engrave / imprint effect
    pub engrave: Option<bool>,
    /// Language override (BCP 47)
    pub language: Option<String>,
    /// Character style reference (by name)
    pub character_style: Option<String>,
}

impl CharacterProperties {
    /// Merge another set of properties on top of this one.
    /// Non-None values in `other` override values in `self`.
    pub fn merge(&self, other: &CharacterProperties) -> CharacterProperties {
        CharacterProperties {
            font_family: other.font_family.clone().or_else(|| self.font_family.clone()),
            font_size: other.font_size.or(self.font_size),
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            underline: other.underline.or(self.underline),
            underline_color: other.underline_color.or(self.underline_color),
            strikethrough: other.strikethrough.or(self.strikethrough),
            color: other.color.or(self.color),
            highlight: other.highlight.or(self.highlight),
            superscript: other.superscript.or(self.superscript),
            subscript: other.subscript.or(self.subscript),
            small_caps: other.small_caps.or(self.small_caps),
            all_caps: other.all_caps.or(self.all_caps),
            hidden: other.hidden.or(self.hidden),
            spacing: other.spacing.or(self.spacing),
            kerning: other.kerning.or(self.kerning),
            position: other.position.or(self.position),
            scale: other.scale.or(self.scale),
            shadow: other.shadow.or(self.shadow),
            outline: other.outline.or(self.outline),
            emboss: other.emboss.or(self.emboss),
            engrave: other.engrave.or(self.engrave),
            language: other.language.clone().or_else(|| self.language.clone()),
            character_style: other.character_style.clone().or_else(|| self.character_style.clone()),
        }
    }
}

/// Paragraph-level formatting properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParagraphProperties {
    /// Paragraph style reference (by name)
    pub paragraph_style: Option<String>,
    /// Text alignment
    pub alignment: Option<Alignment>,
    /// Left indent
    pub indent_left: Option<Twips>,
    /// Right indent
    pub indent_right: Option<Twips>,
    /// First line indent (positive) or hanging indent (negative)
    pub indent_first_line: Option<Twips>,
    /// Space before paragraph
    pub space_before: Option<Twips>,
    /// Space after paragraph
    pub space_after: Option<Twips>,
    /// Line spacing
    pub line_spacing: Option<LineSpacing>,
    /// Keep with next paragraph (don't break between this and next)
    pub keep_with_next: Option<bool>,
    /// Keep lines together (don't break within this paragraph)
    pub keep_together: Option<bool>,
    /// Page break before this paragraph
    pub page_break_before: Option<bool>,
    /// Widow control (minimum lines at top of page)
    pub widow_control: Option<u32>,
    /// Orphan control (minimum lines at bottom of page)
    pub orphan_control: Option<u32>,
    /// Tab stops
    pub tab_stops: Option<Vec<TabStop>>,
    /// Paragraph borders
    pub borders: Option<ParagraphBorders>,
    /// Paragraph background/shading
    pub background: Option<Color>,
    /// Outline level (0 = body text, 1-9 = heading levels)
    pub outline_level: Option<u8>,
    /// List/numbering properties
    pub numbering: Option<NumberingProperties>,
    /// Suppress line numbers
    pub suppress_line_numbers: Option<bool>,
    /// Suppress hyphenation
    pub suppress_hyphenation: Option<bool>,
    /// Text direction
    pub text_direction: Option<TextDirection>,
    /// Default character properties for new text in this paragraph
    pub default_char_props: Option<CharacterProperties>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
    Distribute,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineSpacing {
    /// Multiplier (1.0 = single, 1.5 = one-and-a-half, 2.0 = double)
    Multiple(f64),
    /// Exact spacing in twips
    Exact(Twips),
    /// Minimum spacing in twips
    AtLeast(Twips),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TabStop {
    /// Position from left margin
    pub position: Twips,
    /// Tab alignment
    pub alignment: TabAlignment,
    /// Leader character
    pub leader: TabLeader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabAlignment {
    Left,
    Center,
    Right,
    Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabLeader {
    None,
    Dot,
    Dash,
    Underscore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderlineStyle {
    Single,
    Double,
    Dotted,
    Dashed,
    DashDot,
    DashDotDot,
    Wave,
    Thick,
    Words, // underline words only, not spaces
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikethroughStyle {
    Single,
    Double,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphBorders {
    pub top: Option<BorderLine>,
    pub bottom: Option<BorderLine>,
    pub left: Option<BorderLine>,
    pub right: Option<BorderLine>,
    /// Space between border and text
    pub padding: Option<Twips>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BorderLine {
    pub style: BorderStyle,
    pub width: Twips,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Single,
    Double,
    Dashed,
    Dotted,
    DashDot,
    DashDotDot,
    Thick,
    ThickThinSmall,
    ThinThickSmall,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberingProperties {
    /// Reference to a numbering definition
    pub numbering_id: Option<u32>,
    /// Level within the numbering (0-based, up to 9)
    pub level: u8,
    /// Override the numbering format
    pub format_override: Option<NumberFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFormat {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
    Bullet,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

/// Page-level properties, defining the physical page layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageProperties {
    /// Page width
    pub width: Twips,
    /// Page height
    pub height: Twips,
    /// Page orientation
    pub orientation: PageOrientation,
    /// Margins
    pub margins: PageMargins,
    /// Number of columns
    pub columns: ColumnLayout,
    /// Page numbering
    pub page_numbering: Option<PageNumbering>,
    /// Page background color
    pub background: Option<Color>,
    /// Page borders
    pub borders: Option<ParagraphBorders>,
    /// Gutter position and width (for binding)
    pub gutter: Option<Gutter>,
    /// Mirror margins for facing pages
    pub mirror_margins: bool,
}

impl Default for PageProperties {
    fn default() -> Self {
        Self {
            // US Letter: 8.5" x 11"
            width: Twips::from_inches(8.5),
            height: Twips::from_inches(11.0),
            orientation: PageOrientation::Portrait,
            margins: PageMargins::default(),
            columns: ColumnLayout::Single,
            page_numbering: None,
            background: None,
            borders: None,
            gutter: None,
            mirror_margins: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageMargins {
    pub top: Twips,
    pub bottom: Twips,
    pub left: Twips,
    pub right: Twips,
    pub header: Twips,
    pub footer: Twips,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: Twips::from_inches(1.0),
            bottom: Twips::from_inches(1.0),
            left: Twips::from_inches(1.0),
            right: Twips::from_inches(1.0),
            header: Twips::from_inches(0.5),
            footer: Twips::from_inches(0.5),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnLayout {
    Single,
    Equal {
        count: u32,
        spacing: Twips,
    },
    Custom {
        columns: Vec<ColumnDef>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub width: Twips,
    pub spacing: Twips,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageNumbering {
    pub format: NumberFormat,
    pub start: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Gutter {
    pub width: Twips,
    pub position: GutterPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GutterPosition {
    Left,
    Top,
}
