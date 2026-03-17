//! # rw-command-palette
//!
//! VS Code-style command palette for Rust Writer.
//!
//! Activated via Ctrl+Shift+P, provides:
//! - Fuzzy search over all available commands
//! - Keyboard shortcut display next to each command
//! - Most Recently Used (MRU) ordering
//! - Category filtering (File, Edit, Format, Insert, etc.)
//! - Quick file open mode (Ctrl+P)
//! - Go to line mode (Ctrl+G)

/// A registered command that appears in the palette.
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// Unique command identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Category for grouping
    pub category: String,
    /// Keyboard shortcut (display string)
    pub shortcut: Option<String>,
    /// Description / tooltip
    pub description: Option<String>,
    /// Whether the command is currently available
    pub enabled: bool,
    /// Number of times this command has been used (for MRU sorting)
    pub use_count: u32,
}

/// Command palette mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    /// Search all commands (Ctrl+Shift+P)
    Commands,
    /// Quick file open (Ctrl+P)
    Files,
    /// Go to line (Ctrl+G)
    GoToLine,
    /// Go to heading
    GoToHeading,
}

/// Command palette state.
#[derive(Debug, Clone)]
pub struct PaletteState {
    /// Whether the palette is open
    pub open: bool,
    /// Current mode
    pub mode: PaletteMode,
    /// Current search query
    pub query: String,
    /// Filtered results (indices into command list)
    pub results: Vec<usize>,
    /// Currently highlighted result index
    pub selected: usize,
}

impl Default for PaletteState {
    fn default() -> Self {
        Self {
            open: false,
            mode: PaletteMode::Commands,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
        }
    }
}

/// Simple fuzzy matching: returns a score if the query matches the target.
/// Higher scores indicate better matches.
pub fn fuzzy_match(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    // Exact substring match gets highest score
    if target_lower.contains(&query_lower) {
        let position = target_lower.find(&query_lower).unwrap();
        return Some(1000 - position as i32);
    }

    // Fuzzy character-by-character match
    let mut query_chars = query_lower.chars();
    let mut current = query_chars.next()?;
    let mut score = 0i32;
    let mut consecutive = 0i32;

    for (i, ch) in target_lower.chars().enumerate() {
        if ch == current {
            score += 10 + consecutive * 5;
            if i == 0 { score += 20; } // bonus for matching at start
            consecutive += 1;
            match query_chars.next() {
                Some(next) => current = next,
                None => return Some(score),
            }
        } else {
            consecutive = 0;
        }
    }

    None // not all query characters matched
}
