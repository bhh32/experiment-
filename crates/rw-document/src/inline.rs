use crate::properties::CharacterProperties;
use crate::text_run::TextRun;
use crate::ElementId;
use serde::{Deserialize, Serialize};

/// Inline content elements that appear within a paragraph.
///
/// These are the building blocks of paragraph content — text runs,
/// images, fields, footnote references, bookmarks, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Inline {
    /// A run of text with uniform formatting
    Text(TextRun),

    /// An inline image
    Image(InlineImage),

    /// A field (page number, date, cross-reference, etc.)
    Field(FieldRef),

    /// A footnote or endnote reference
    NoteRef(NoteRef),

    /// Start of a bookmark range
    BookmarkStart(BookmarkMark),

    /// End of a bookmark range
    BookmarkEnd(BookmarkMark),

    /// Start of a comment range
    CommentRangeStart(CommentMark),

    /// End of a comment range
    CommentRangeEnd(CommentMark),

    /// Start of a tracked change range
    ChangeStart(ChangeMark),

    /// End of a tracked change range
    ChangeEnd(ChangeMark),

    /// A break (line, column, or page)
    Break(BreakType),

    /// A tab character
    Tab,

    /// A non-breaking space
    NonBreakingSpace,

    /// A soft hyphen
    SoftHyphen,

    /// A hyperlink wrapping other inline content
    Hyperlink(Hyperlink),

    /// A Ruby annotation (for CJK text)
    Ruby(RubyAnnotation),
}

/// An image displayed inline within text flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineImage {
    pub id: ElementId,
    /// Reference to the image resource (path, embedded data ID, etc.)
    pub source: ImageSource,
    /// Display width
    pub width: crate::Twips,
    /// Display height
    pub height: crate::Twips,
    /// Alt text for accessibility
    pub alt_text: Option<String>,
    /// Title / tooltip
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSource {
    /// Path to an external file
    File(String),
    /// Embedded binary data with MIME type
    Embedded {
        data_id: String,
        mime_type: String,
    },
    /// URL to an online image
    Url(String),
}

/// A reference to a field defined in rw-fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRef {
    pub id: ElementId,
    pub field_type: FieldType,
    /// Cached display value
    pub cached_value: Option<String>,
    /// Character properties for the field display
    pub properties: CharacterProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    PageNumber,
    PageCount,
    Date { format: String },
    Time { format: String },
    FileName,
    Author,
    Title,
    Subject,
    Custom { name: String, value: String },
    CrossReference { bookmark_name: String, ref_type: CrossRefType },
    Sequence { name: String },
    MergeField { field_name: String },
    If { condition: String, true_text: String, false_text: String },
    TableOfContents,
    Index,
    Bibliography,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CrossRefType {
    PageNumber,
    ParagraphNumber,
    Text,
    Above,
    Below,
}

/// Reference to a footnote or endnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRef {
    pub id: ElementId,
    pub note_type: NoteType,
    pub note_id: ElementId,
    pub properties: CharacterProperties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteType {
    Footnote,
    Endnote,
}

/// A bookmark marker (start or end).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkMark {
    pub id: ElementId,
    pub name: String,
}

/// A comment range marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentMark {
    pub id: ElementId,
    pub comment_id: ElementId,
}

/// A tracked-change range marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeMark {
    pub id: ElementId,
    pub change_id: ElementId,
}

/// Type of break within a paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakType {
    /// Line break (soft return)
    Line,
    /// Column break
    Column,
    /// Page break
    Page,
}

/// A hyperlink wrapping inline content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperlink {
    pub id: ElementId,
    /// URL or internal bookmark target
    pub target: HyperlinkTarget,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// The inline content displayed as the link
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HyperlinkTarget {
    Url(String),
    Bookmark(String),
    Email { address: String, subject: Option<String> },
}

/// Ruby annotation for CJK text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubyAnnotation {
    pub id: ElementId,
    pub base_text: String,
    pub annotation_text: String,
    pub properties: CharacterProperties,
}
