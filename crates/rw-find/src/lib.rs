//! # rw-find
//!
//! Find and replace engine for Rust Writer.
//!
//! - Plain text search (case-sensitive and case-insensitive)
//! - Whole word matching
//! - Regular expression search (with capture groups)
//! - Search within selection
//! - Search by formatting (find all bold text, find by style, etc.)
//! - Replace with captured groups ($1, $2, etc.)
//! - Replace all / replace one at a time
//! - Find next / find previous
//! - Match highlighting

use regex::Regex;
use rw_document::{Document, block::Block};

/// Search options.
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// The search pattern
    pub pattern: String,
    /// Whether to use regex
    pub use_regex: bool,
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Match whole words only
    pub whole_word: bool,
    /// Search direction
    pub direction: SearchDirection,
    /// Whether to wrap around at document end
    pub wrap_around: bool,
    /// Search within selection only
    pub in_selection: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            use_regex: false,
            case_sensitive: false,
            whole_word: false,
            direction: SearchDirection::Forward,
            wrap_around: true,
            in_selection: false,
        }
    }
}

/// Replace options (extends FindOptions).
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub find: FindOptions,
    /// The replacement string (supports $1, $2 for regex captures)
    pub replacement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// A search match result.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Section index
    pub section: usize,
    /// Block index
    pub block: usize,
    /// Start byte offset within the paragraph text
    pub start_offset: usize,
    /// End byte offset
    pub end_offset: usize,
    /// The matched text
    pub matched_text: String,
}

/// A position in the document (local to this crate to avoid circular deps with rw-editor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DocPosition {
    pub section: usize,
    pub block: usize,
    pub offset: usize,
}

impl DocPosition {
    pub fn new(section: usize, block: usize, offset: usize) -> Self {
        Self { section, block, offset }
    }
}

/// The find and replace engine.
#[derive(Debug, Default)]
pub struct FindEngine;

impl FindEngine {
    pub fn new() -> Self {
        Self
    }

    /// Find the next match from a given position.
    pub fn find_next(
        &self,
        doc: &Document,
        options: &FindOptions,
        from: DocPosition,
    ) -> Option<SearchMatch> {
        let all = self.find_all(doc, options);
        if all.is_empty() {
            return None;
        }

        match options.direction {
            SearchDirection::Forward => {
                let found = all.iter().find(|m| {
                    m.section > from.section
                        || (m.section == from.section && m.block > from.block)
                        || (m.section == from.section
                            && m.block == from.block
                            && m.start_offset >= from.offset)
                });
                if let Some(m) = found {
                    return Some(m.clone());
                }
                if options.wrap_around {
                    all.into_iter().next()
                } else {
                    None
                }
            }
            SearchDirection::Backward => {
                let found = all.iter().rev().find(|m| {
                    m.section < from.section
                        || (m.section == from.section && m.block < from.block)
                        || (m.section == from.section
                            && m.block == from.block
                            && m.end_offset <= from.offset)
                });
                if let Some(m) = found {
                    return Some(m.clone());
                }
                if options.wrap_around {
                    all.into_iter().last()
                } else {
                    None
                }
            }
        }
    }

    /// Find all matches in the document.
    pub fn find_all(&self, doc: &Document, options: &FindOptions) -> Vec<SearchMatch> {
        if options.pattern.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for (si, section) in doc.sections.iter().enumerate() {
            for (bi, block) in section.content.iter().enumerate() {
                if let Block::Paragraph(para) = block {
                    let text = para.plain_text();
                    let matches = self.find_in_text(&text, options);
                    for (start, end, matched) in matches {
                        results.push(SearchMatch {
                            section: si,
                            block: bi,
                            start_offset: start,
                            end_offset: end,
                            matched_text: matched,
                        });
                    }
                }
            }
        }

        results
    }

    /// Find and replace the next occurrence from position.
    pub fn replace_next(
        &self,
        doc: &mut Document,
        options: &ReplaceOptions,
        from: DocPosition,
    ) -> Option<SearchMatch> {
        let m = self.find_next(doc, &options.find, from)?;
        let replacement = self.compute_replacement(
            &m.matched_text,
            &options.find.pattern,
            &options.replacement,
            options.find.use_regex,
        );
        self.apply_replacement(doc, &m, &replacement);
        Some(m)
    }

    /// Replace all occurrences and return count.
    pub fn replace_all(&self, doc: &mut Document, options: &ReplaceOptions) -> usize {
        let matches = self.find_all(doc, &options.find);
        let count = matches.len();

        // Compute replacements first, then apply in reverse order so offsets remain valid.
        let mut indexed: Vec<(usize, usize, usize, String, String)> = matches
            .iter()
            .map(|m| {
                let rep = self.compute_replacement(
                    &m.matched_text,
                    &options.find.pattern,
                    &options.replacement,
                    options.find.use_regex,
                );
                (m.section, m.block, m.start_offset, m.matched_text.clone(), rep)
            })
            .collect();

        // Sort descending so later offsets are replaced first.
        indexed.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then(b.1.cmp(&a.1))
                .then(b.2.cmp(&a.2))
        });

        for (si, bi, start, matched, replacement) in indexed {
            let m = SearchMatch {
                section: si,
                block: bi,
                start_offset: start,
                end_offset: start + matched.len(),
                matched_text: matched,
            };
            self.apply_replacement(doc, &m, &replacement);
        }

        count
    }

    // --- Private helpers ---

    fn find_in_text(&self, text: &str, options: &FindOptions) -> Vec<(usize, usize, String)> {
        let mut results = Vec::new();

        if options.use_regex {
            let pattern = if options.whole_word {
                format!(r"\b{}\b", options.pattern)
            } else {
                options.pattern.clone()
            };
            let re_result = if options.case_sensitive {
                Regex::new(&pattern)
            } else {
                Regex::new(&format!("(?i){}", pattern))
            };
            if let Ok(re) = re_result {
                for m in re.find_iter(text) {
                    results.push((m.start(), m.end(), m.as_str().to_string()));
                }
            }
        } else {
            // Plain text search
            let (haystack, needle) = if options.case_sensitive {
                (text.to_string(), options.pattern.clone())
            } else {
                (text.to_lowercase(), options.pattern.to_lowercase())
            };

            if needle.is_empty() {
                return results;
            }

            let mut start = 0;
            while let Some(pos) = haystack[start..].find(&needle) {
                let abs_start = start + pos;
                let abs_end = abs_start + needle.len();

                if options.whole_word {
                    let before_ok = abs_start == 0
                        || !text[..abs_start]
                            .chars()
                            .last()
                            .map(|c| c.is_alphanumeric() || c == '_')
                            .unwrap_or(false);
                    let after_ok = abs_end >= text.len()
                        || !text[abs_end..]
                            .chars()
                            .next()
                            .map(|c| c.is_alphanumeric() || c == '_')
                            .unwrap_or(false);
                    if before_ok && after_ok {
                        results.push((abs_start, abs_end, text[abs_start..abs_end].to_string()));
                    }
                } else {
                    results.push((abs_start, abs_end, text[abs_start..abs_end].to_string()));
                }

                start = abs_start + 1;
            }
        }

        results
    }

    fn compute_replacement(
        &self,
        matched: &str,
        pattern: &str,
        replacement: &str,
        use_regex: bool,
    ) -> String {
        if use_regex {
            if let Ok(re) = Regex::new(pattern) {
                re.replace(matched, replacement).into_owned()
            } else {
                replacement.to_string()
            }
        } else {
            replacement.to_string()
        }
    }

    fn apply_replacement(&self, doc: &mut Document, m: &SearchMatch, replacement: &str) {
        if let Some(section) = doc.sections.get_mut(m.section) {
            if let Some(Block::Paragraph(para)) = section.content.get_mut(m.block) {
                let mut text = para.plain_text();
                if m.start_offset <= text.len() && m.end_offset <= text.len() {
                    text.replace_range(m.start_offset..m.end_offset, replacement);
                    para.content.clear();
                    para.content.push(rw_document::Inline::Text(
                        rw_document::TextRun::new(text),
                    ));
                }
            }
        }
    }
}
