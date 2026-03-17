//! # rw-spell
//!
//! Spell checking for Rust Writer using Hunspell.
//!
//! - Real-time spell checking with red underline indicators
//! - Suggestion generation for misspelled words
//! - Custom user dictionary (add word / ignore word)
//! - Auto-correct (common typo corrections)
//! - Multiple language support (via Hunspell dictionaries)
//! - Per-paragraph language detection/override

/// A spell checking error found in the document.
#[derive(Debug, Clone)]
pub struct SpellError {
    /// The misspelled word
    pub word: String,
    /// Suggested corrections
    pub suggestions: Vec<String>,
    /// Position in the document
    pub section: usize,
    pub block: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Auto-correct entry.
#[derive(Debug, Clone)]
pub struct AutoCorrectEntry {
    pub trigger: String,
    pub replacement: String,
}

/// Spell checker configuration.
#[derive(Debug, Clone)]
pub struct SpellConfig {
    /// Whether spell checking is enabled
    pub enabled: bool,
    /// Whether to check as you type
    pub check_as_you_type: bool,
    /// Whether to auto-correct
    pub auto_correct: bool,
    /// Default language
    pub default_language: String,
    /// Words to ignore (user dictionary)
    pub ignored_words: Vec<String>,
    /// Auto-correct entries
    pub auto_correct_entries: Vec<AutoCorrectEntry>,
}

impl Default for SpellConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_as_you_type: true,
            auto_correct: true,
            default_language: "en_US".to_string(),
            ignored_words: Vec::new(),
            auto_correct_entries: default_auto_corrections(),
        }
    }
}

/// Common auto-correction entries.
fn default_auto_corrections() -> Vec<AutoCorrectEntry> {
    vec![
        AutoCorrectEntry { trigger: "teh".to_string(), replacement: "the".to_string() },
        AutoCorrectEntry { trigger: "adn".to_string(), replacement: "and".to_string() },
        AutoCorrectEntry { trigger: "dont".to_string(), replacement: "don't".to_string() },
        AutoCorrectEntry { trigger: "cant".to_string(), replacement: "can't".to_string() },
        AutoCorrectEntry { trigger: "wont".to_string(), replacement: "won't".to_string() },
        AutoCorrectEntry { trigger: "its".to_string(), replacement: "it's".to_string() },
        AutoCorrectEntry { trigger: "im".to_string(), replacement: "I'm".to_string() },
        AutoCorrectEntry { trigger: "ive".to_string(), replacement: "I've".to_string() },
        AutoCorrectEntry { trigger: "thier".to_string(), replacement: "their".to_string() },
        AutoCorrectEntry { trigger: "recieve".to_string(), replacement: "receive".to_string() },
    ]
}
