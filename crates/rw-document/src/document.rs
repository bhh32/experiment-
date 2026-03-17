use crate::block::Block;
use crate::inline::NoteType;
use crate::metadata::Metadata;
use crate::paragraph::Paragraph;
use crate::section::Section;
use crate::ElementId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root document structure — the complete representation of a
/// word processing document in memory.
///
/// This is the central data structure of Rust Writer. All editing
/// operations, format conversions, and rendering work with this model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document metadata (title, author, dates, etc.)
    pub metadata: Metadata,

    /// The document body, organized into sections.
    /// Each section can have its own page layout.
    pub sections: Vec<Section>,

    /// Footnotes, keyed by their ElementId
    pub footnotes: HashMap<ElementId, Footnote>,

    /// Endnotes, keyed by their ElementId
    pub endnotes: HashMap<ElementId, Footnote>,

    /// Comments/annotations, keyed by their ElementId
    pub comments: HashMap<ElementId, Comment>,

    /// Numbering definitions (for bulleted/numbered lists)
    pub numbering_definitions: Vec<NumberingDefinition>,

    /// Embedded resources (images, etc.)
    pub resources: ResourceStore,

    /// Document-level default properties
    pub defaults: DocumentDefaults,
}

impl Document {
    /// Create a new, empty document with default settings.
    pub fn new() -> Self {
        let mut doc = Self {
            metadata: Metadata::default(),
            sections: Vec::new(),
            footnotes: HashMap::new(),
            endnotes: HashMap::new(),
            comments: HashMap::new(),
            numbering_definitions: Vec::new(),
            resources: ResourceStore::default(),
            defaults: DocumentDefaults::default(),
        };
        // Every document starts with at least one section containing one empty paragraph
        let mut section = Section::new();
        section.content.push(Block::Paragraph(Paragraph::new()));
        doc.sections.push(section);
        doc
    }

    /// Get a flat iterator over all blocks in the document.
    pub fn all_blocks(&self) -> impl Iterator<Item = &Block> {
        self.sections.iter().flat_map(|s| s.content.iter())
    }

    /// Get a mutable flat iterator over all blocks in the document.
    pub fn all_blocks_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        self.sections.iter_mut().flat_map(|s| s.content.iter_mut())
    }

    /// Get a flat iterator over all paragraphs in the document.
    pub fn all_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.all_blocks().filter_map(|b| {
            if let Block::Paragraph(p) = b {
                Some(p)
            } else {
                None
            }
        })
    }

    /// Extract the full plain text of the document.
    pub fn plain_text(&self) -> String {
        self.all_paragraphs()
            .map(|p| p.plain_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Count words in the document.
    pub fn word_count(&self) -> usize {
        let text = self.plain_text();
        text.split_whitespace().count()
    }

    /// Add a footnote and return its ID.
    pub fn add_footnote(&mut self, content: Vec<Block>) -> ElementId {
        let id = ElementId::new();
        self.footnotes.insert(id, Footnote {
            id,
            note_type: NoteType::Footnote,
            content,
        });
        id
    }

    /// Add an endnote and return its ID.
    pub fn add_endnote(&mut self, content: Vec<Block>) -> ElementId {
        let id = ElementId::new();
        self.endnotes.insert(id, Footnote {
            id,
            note_type: NoteType::Endnote,
            content,
        });
        id
    }

    /// Add a comment and return its ID.
    pub fn add_comment(&mut self, author: String, text: String) -> ElementId {
        let id = ElementId::new();
        self.comments.insert(id, Comment {
            id,
            author,
            date: chrono::Utc::now(),
            content: vec![Block::Paragraph(Paragraph::with_text(text))],
            resolved: false,
            replies: Vec::new(),
        });
        id
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// A footnote or endnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    pub id: ElementId,
    pub note_type: NoteType,
    pub content: Vec<Block>,
}

/// A comment/annotation attached to a range of document content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: ElementId,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub content: Vec<Block>,
    pub resolved: bool,
    pub replies: Vec<Comment>,
}

/// A numbering definition (for bulleted and numbered lists).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberingDefinition {
    pub id: u32,
    pub name: Option<String>,
    /// Up to 10 levels (0-9)
    pub levels: Vec<NumberingLevel>,
}

/// A single level in a numbering definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberingLevel {
    /// Level index (0-based)
    pub level: u8,
    /// Number format
    pub format: crate::properties::NumberFormat,
    /// Text template (e.g., "%1." or "(%1)")
    pub level_text: String,
    /// Starting number
    pub start: u32,
    /// Indentation
    pub indent: crate::Twips,
    /// Character following the number
    pub suffix: NumberingSuffix,
    /// Character style for the number
    pub char_style: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingSuffix {
    Tab,
    Space,
    Nothing,
}

/// Storage for embedded resources (images, fonts, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceStore {
    /// Embedded binary resources keyed by data_id
    pub resources: HashMap<String, EmbeddedResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedResource {
    /// Unique identifier
    pub data_id: String,
    /// MIME type
    pub mime_type: String,
    /// Original filename (if known)
    pub filename: Option<String>,
    /// The binary data
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
}

/// Serde helper for base64-encoding binary data.
mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = data.iter().fold(String::new(), |mut acc, byte| {
            use std::fmt::Write;
            write!(acc, "{:02x}", byte).unwrap();
            acc
        });
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(D::Error::custom))
            .collect()
    }
}

/// Document-wide default formatting properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDefaults {
    /// Default character properties (applied when no style overrides)
    pub default_char_properties: crate::properties::CharacterProperties,
    /// Default paragraph properties
    pub default_para_properties: crate::properties::ParagraphProperties,
    /// Default page properties
    pub default_page_properties: crate::properties::PageProperties,
}

impl Default for DocumentDefaults {
    fn default() -> Self {
        let mut char_props = crate::properties::CharacterProperties::default();
        char_props.font_family = Some("Liberation Serif".to_string());
        char_props.font_size = Some(24); // 12pt in half-points
        char_props.color = Some(crate::Color::BLACK);

        let mut para_props = crate::properties::ParagraphProperties::default();
        para_props.alignment = Some(crate::properties::Alignment::Left);
        para_props.line_spacing = Some(crate::properties::LineSpacing::Multiple(1.15));
        para_props.space_after = Some(crate::Twips::from_points(8.0));

        Self {
            default_char_properties: char_props,
            default_para_properties: para_props,
            default_page_properties: crate::properties::PageProperties::default(),
        }
    }
}
