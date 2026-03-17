use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document metadata — properties like title, author, creation date, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Document title
    pub title: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document description / comments
    pub description: Option<String>,
    /// Keywords for the document
    pub keywords: Vec<String>,
    /// Creation timestamp
    pub created: DateTime<Utc>,
    /// Last modification timestamp
    pub modified: DateTime<Utc>,
    /// Last printed timestamp
    pub last_printed: Option<DateTime<Utc>>,
    /// Revision number
    pub revision: u32,
    /// Total editing time in seconds
    pub editing_time: u64,
    /// Application that created the document
    pub generator: String,
    /// Language of the document (BCP 47 tag, e.g., "en-US")
    pub language: Option<String>,
    /// Custom properties
    pub custom_properties: HashMap<String, MetadataValue>,
    /// Document statistics
    pub statistics: DocumentStatistics,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            title: None,
            subject: None,
            author: None,
            description: None,
            keywords: Vec::new(),
            created: now,
            modified: now,
            last_printed: None,
            revision: 1,
            editing_time: 0,
            generator: format!("Rust Writer {}", env!("CARGO_PKG_VERSION")),
            language: Some("en-US".to_string()),
            custom_properties: HashMap::new(),
            statistics: DocumentStatistics::default(),
        }
    }
}

/// A typed metadata value for custom properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Date(DateTime<Utc>),
}

/// Document statistics (word count, page count, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentStatistics {
    pub page_count: u32,
    pub paragraph_count: u32,
    pub word_count: u32,
    pub character_count: u32,
    pub character_count_with_spaces: u32,
    pub line_count: u32,
    pub table_count: u32,
    pub image_count: u32,
}
