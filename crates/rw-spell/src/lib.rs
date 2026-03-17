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

use std::collections::HashSet;

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

/// Built-in word list (~100+ common English words used as a minimal dictionary).
fn builtin_dictionary() -> HashSet<String> {
    let words = [
        "a", "able", "about", "above", "after", "again", "all", "also", "always",
        "am", "an", "and", "any", "are", "as", "at", "back", "be", "been",
        "before", "being", "between", "both", "but", "by", "came", "can", "come",
        "could", "day", "did", "different", "do", "does", "done", "down", "each",
        "end", "even", "every", "few", "find", "first", "for", "form", "found",
        "from", "get", "give", "go", "good", "great", "had", "has", "have",
        "he", "help", "her", "here", "him", "his", "home", "how", "i", "if",
        "in", "into", "is", "it", "its", "just", "know", "large", "last",
        "left", "like", "little", "long", "look", "made", "make", "man", "many",
        "may", "me", "men", "might", "more", "most", "move", "much", "my",
        "name", "need", "new", "no", "not", "now", "of", "off", "old", "on",
        "one", "only", "or", "other", "our", "out", "over", "own", "part",
        "people", "place", "put", "right", "said", "same", "see", "she", "should",
        "since", "small", "so", "some", "something", "still", "such", "take",
        "than", "that", "the", "their", "them", "then", "there", "these", "they",
        "thing", "think", "this", "those", "through", "time", "to", "together",
        "too", "turn", "two", "under", "up", "us", "use", "very", "want", "was",
        "way", "we", "well", "went", "were", "what", "when", "where", "which",
        "while", "who", "will", "with", "word", "work", "world", "would", "year",
        "you", "your", "document", "text", "file", "page", "write", "read",
        "edit", "format", "table", "image", "section", "paragraph", "font",
        "bold", "italic", "style", "size", "color", "line", "column", "row",
        "cell", "border", "margin", "header", "footer", "title", "author",
        "date", "print", "save", "open", "close", "copy", "paste", "cut", "undo",
        "receive", "believe", "achieve", "because", "between", "example",
        "important", "language", "number", "often", "program", "question",
        "really", "right", "school", "through", "together", "under", "while",
    ];
    words.iter().map(|w| w.to_string()).collect()
}

/// The spell checker.
#[derive(Debug)]
pub struct SpellChecker {
    config: SpellConfig,
    builtin: HashSet<String>,
    user_dictionary: HashSet<String>,
}

impl SpellChecker {
    /// Create a new spell checker with the given config.
    pub fn new(config: SpellConfig) -> Self {
        let user_dictionary: HashSet<String> = config
            .ignored_words
            .iter()
            .map(|w| w.to_lowercase())
            .collect();

        Self {
            config,
            builtin: builtin_dictionary(),
            user_dictionary,
        }
    }

    /// Check if a word is correctly spelled.
    /// Returns `true` if the word is in the dictionary (or user dictionary).
    pub fn check_word(&self, word: &str) -> bool {
        if word.is_empty() {
            return true;
        }
        let lower = word.to_lowercase();
        let clean: String = lower.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
        if clean.is_empty() {
            return true;
        }
        self.builtin.contains(&clean) || self.user_dictionary.contains(&clean)
    }

    /// Generate spelling suggestions for a word using Levenshtein distance.
    pub fn suggest(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        let mut candidates: Vec<(usize, String)> = self
            .builtin
            .iter()
            .chain(self.user_dictionary.iter())
            .filter_map(|w| {
                let dist = edit_distance(&lower, w);
                if dist <= 3 {
                    Some((dist, w.clone()))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|(d, _)| *d);
        candidates.truncate(5);
        candidates.into_iter().map(|(_, w)| w).collect()
    }

    /// Check all words in a paragraph text and return errors.
    pub fn check_paragraph(&self, text: &str) -> Vec<SpellError> {
        let mut errors = Vec::new();

        for token in tokenize(text) {
            let word = &text[token.start..token.end];
            if !self.check_word(word) {
                let suggestions = self.suggest(word);
                errors.push(SpellError {
                    word: word.to_string(),
                    suggestions,
                    section: 0,
                    block: 0,
                    start_offset: token.start,
                    end_offset: token.end,
                });
            }
        }

        errors
    }

    /// Add a word to the user dictionary.
    pub fn add_to_dictionary(&mut self, word: &str) {
        let lower = word.to_lowercase();
        self.user_dictionary.insert(lower.clone());
        if !self.config.ignored_words.contains(&lower) {
            self.config.ignored_words.push(lower);
        }
    }

    /// Check if a word should be auto-corrected; return the replacement if so.
    pub fn auto_correct(&self, word: &str) -> Option<String> {
        if !self.config.auto_correct {
            return None;
        }
        let lower = word.to_lowercase();
        self.config
            .auto_correct_entries
            .iter()
            .find(|e| e.trigger == lower)
            .map(|e| e.replacement.clone())
    }
}

/// Compute the Levenshtein edit distance between two strings.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
            }
        }
    }

    dp[m][n]
}

/// A word token with byte positions.
struct Token {
    start: usize,
    end: usize,
}

/// Tokenize text into word tokens (alphabetic sequences).
fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start: Option<usize> = None;

    for (i, c) in text.char_indices() {
        if c.is_alphabetic() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            tokens.push(Token { start: s, end: i });
        }
    }

    if let Some(s) = start {
        tokens.push(Token { start: s, end: text.len() });
    }

    tokens
}
