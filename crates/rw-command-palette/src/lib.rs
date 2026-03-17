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

// ---------------------------------------------------------------------------
// CommandRegistry
// ---------------------------------------------------------------------------

/// Registry of all available commands.
pub struct CommandRegistry {
    commands: Vec<PaletteCommand>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    /// Create a registry pre-populated with all built-in commands.
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        for cmd in default_commands() {
            reg.register(cmd);
        }
        reg
    }

    /// Register a command. If a command with the same id already exists,
    /// it is replaced.
    pub fn register(&mut self, cmd: PaletteCommand) {
        if let Some(pos) = self.commands.iter().position(|c| c.id == cmd.id) {
            self.commands[pos] = cmd;
        } else {
            self.commands.push(cmd);
        }
    }

    /// Remove a command by id.
    pub fn unregister(&mut self, id: &str) {
        self.commands.retain(|c| c.id != id);
    }

    /// Look up a command by id.
    pub fn get(&self, id: &str) -> Option<&PaletteCommand> {
        self.commands.iter().find(|c| c.id == id)
    }

    /// Search for commands matching `query`.
    ///
    /// Results are sorted by:
    /// 1. Fuzzy score (descending)
    /// 2. Use count (descending) for equal scores
    /// 3. Label alphabetically for tie-breaking
    pub fn search(&self, query: &str) -> Vec<&PaletteCommand> {
        let mut scored: Vec<(&PaletteCommand, i32)> = self
            .commands
            .iter()
            .filter(|c| c.enabled)
            .filter_map(|c| {
                // Match against label, category, and description
                let label_score = fuzzy_match(query, &c.label);
                let cat_score = fuzzy_match(query, &c.category).map(|s| s / 2);
                let desc_score = c
                    .description
                    .as_deref()
                    .and_then(|d| fuzzy_match(query, d))
                    .map(|s| s / 3);

                let best = label_score
                    .into_iter()
                    .chain(cat_score)
                    .chain(desc_score)
                    .max()?;

                Some((c, best + c.use_count as i32))
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.0.use_count.cmp(&a.0.use_count))
                .then_with(|| a.0.label.cmp(&b.0.label))
        });

        scored.into_iter().map(|(c, _)| c).collect()
    }

    /// Record that a command was used (increments use_count for MRU ordering).
    pub fn record_use(&mut self, id: &str) {
        if let Some(cmd) = self.commands.iter_mut().find(|c| c.id == id) {
            cmd.use_count = cmd.use_count.saturating_add(1);
        }
    }

    /// Return all commands in a given category.
    pub fn by_category(&self, category: &str) -> Vec<&PaletteCommand> {
        self.commands.iter().filter(|c| c.category == category).collect()
    }

    /// Return all registered commands.
    pub fn all(&self) -> &[PaletteCommand] {
        &self.commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Default commands
// ---------------------------------------------------------------------------

/// Build the complete list of built-in commands.
pub fn default_commands() -> Vec<PaletteCommand> {
    let mut cmds: Vec<PaletteCommand> = Vec::new();

    // ----- File -----
    cmds.push(cmd("file.new", "New Document", "File", Some("Ctrl+N"), Some("Create a new, empty document")));
    cmds.push(cmd("file.open", "Open...", "File", Some("Ctrl+O"), Some("Open a document from disk")));
    cmds.push(cmd("file.save", "Save", "File", Some("Ctrl+S"), Some("Save the current document")));
    cmds.push(cmd("file.save_as", "Save As...", "File", Some("Ctrl+Shift+S"), Some("Save with a new name or format")));
    cmds.push(cmd("file.close", "Close", "File", Some("Ctrl+W"), Some("Close the current document")));
    cmds.push(cmd("file.print", "Print...", "File", Some("Ctrl+P"), Some("Print the current document")));
    cmds.push(cmd("file.export_pdf", "Export as PDF", "File", None, Some("Export the document as PDF")));
    cmds.push(cmd("file.export_html", "Export as HTML", "File", None, None));
    cmds.push(cmd("file.properties", "Document Properties", "File", None, Some("View and edit document metadata")));

    // ----- Edit -----
    cmds.push(cmd("edit.undo", "Undo", "Edit", Some("Ctrl+Z"), Some("Undo the last action")));
    cmds.push(cmd("edit.redo", "Redo", "Edit", Some("Ctrl+Y"), Some("Redo the last undone action")));
    cmds.push(cmd("edit.cut", "Cut", "Edit", Some("Ctrl+X"), Some("Cut selected text to clipboard")));
    cmds.push(cmd("edit.copy", "Copy", "Edit", Some("Ctrl+C"), Some("Copy selected text to clipboard")));
    cmds.push(cmd("edit.paste", "Paste", "Edit", Some("Ctrl+V"), Some("Paste from clipboard")));
    cmds.push(cmd("edit.paste_plain", "Paste Plain Text", "Edit", Some("Ctrl+Shift+V"), None));
    cmds.push(cmd("edit.select_all", "Select All", "Edit", Some("Ctrl+A"), Some("Select all document content")));
    cmds.push(cmd("edit.find", "Find...", "Edit", Some("Ctrl+F"), Some("Find text in the document")));
    cmds.push(cmd("edit.replace", "Find & Replace...", "Edit", Some("Ctrl+H"), Some("Find and replace text")));
    cmds.push(cmd("edit.go_to_page", "Go to Page...", "Edit", Some("Ctrl+G"), Some("Jump to a specific page")));
    cmds.push(cmd("edit.go_to_heading", "Go to Heading...", "Edit", None, Some("Jump to a heading")));

    // ----- Format -----
    cmds.push(cmd("format.bold", "Bold", "Format", Some("Ctrl+B"), Some("Toggle bold formatting")));
    cmds.push(cmd("format.italic", "Italic", "Format", Some("Ctrl+I"), Some("Toggle italic formatting")));
    cmds.push(cmd("format.underline", "Underline", "Format", Some("Ctrl+U"), Some("Toggle underline formatting")));
    cmds.push(cmd("format.strikethrough", "Strikethrough", "Format", None, Some("Toggle strikethrough")));
    cmds.push(cmd("format.superscript", "Superscript", "Format", Some("Ctrl+Shift+="), None));
    cmds.push(cmd("format.subscript", "Subscript", "Format", Some("Ctrl+="), None));
    cmds.push(cmd("format.clear", "Clear Formatting", "Format", Some("Ctrl+Space"), Some("Remove all direct formatting")));
    cmds.push(cmd("format.align_left", "Align Left", "Format", Some("Ctrl+L"), None));
    cmds.push(cmd("format.align_center", "Center", "Format", Some("Ctrl+E"), None));
    cmds.push(cmd("format.align_right", "Align Right", "Format", Some("Ctrl+R"), None));
    cmds.push(cmd("format.justify", "Justify", "Format", Some("Ctrl+J"), None));
    cmds.push(cmd("format.indent_increase", "Increase Indent", "Format", Some("Tab"), None));
    cmds.push(cmd("format.indent_decrease", "Decrease Indent", "Format", Some("Shift+Tab"), None));
    cmds.push(cmd("format.paragraph_dialog", "Paragraph Settings...", "Format", None, Some("Open the paragraph settings dialog")));
    cmds.push(cmd("format.font_dialog", "Font Settings...", "Format", Some("Ctrl+D"), Some("Open the font settings dialog")));
    cmds.push(cmd("format.styles_dialog", "Manage Styles...", "Format", None, Some("Open the styles pane")));

    // ----- Insert -----
    cmds.push(cmd("insert.page_break", "Insert Page Break", "Insert", Some("Ctrl+Enter"), None));
    cmds.push(cmd("insert.column_break", "Insert Column Break", "Insert", Some("Ctrl+Shift+Enter"), None));
    cmds.push(cmd("insert.table", "Insert Table...", "Insert", None, Some("Insert a table")));
    cmds.push(cmd("insert.image", "Insert Image...", "Insert", None, Some("Insert an image")));
    cmds.push(cmd("insert.hyperlink", "Insert Hyperlink...", "Insert", Some("Ctrl+K"), None));
    cmds.push(cmd("insert.footnote", "Insert Footnote", "Insert", Some("Alt+Ctrl+F"), None));
    cmds.push(cmd("insert.endnote", "Insert Endnote", "Insert", Some("Alt+Ctrl+D"), None));
    cmds.push(cmd("insert.comment", "Insert Comment", "Insert", Some("Ctrl+Alt+M"), None));
    cmds.push(cmd("insert.symbol", "Insert Symbol...", "Insert", None, Some("Insert a special character")));
    cmds.push(cmd("insert.date_time", "Insert Date and Time...", "Insert", None, None));
    cmds.push(cmd("insert.field", "Insert Field...", "Insert", None, Some("Insert a document field")));
    cmds.push(cmd("insert.toc", "Insert Table of Contents", "Insert", None, None));
    cmds.push(cmd("insert.header", "Edit Header", "Insert", None, None));
    cmds.push(cmd("insert.footer", "Edit Footer", "Insert", None, None));

    // ----- View -----
    cmds.push(cmd("view.print_layout", "Print Layout View", "View", None, None));
    cmds.push(cmd("view.web_layout", "Web Layout View", "View", None, None));
    cmds.push(cmd("view.outline", "Outline View", "View", None, None));
    cmds.push(cmd("view.draft", "Draft View", "View", None, None));
    cmds.push(cmd("view.read_mode", "Read Mode", "View", None, None));
    cmds.push(cmd("view.zoom_in", "Zoom In", "View", Some("Ctrl++"), None));
    cmds.push(cmd("view.zoom_out", "Zoom Out", "View", Some("Ctrl+-"), None));
    cmds.push(cmd("view.zoom_100", "Zoom 100%", "View", Some("Ctrl+0"), None));
    cmds.push(cmd("view.toggle_ruler", "Toggle Ruler", "View", None, None));
    cmds.push(cmd("view.toggle_navigation", "Toggle Navigation Pane", "View", Some("Ctrl+F5"), None));
    cmds.push(cmd("view.toggle_formatting_marks", "Show/Hide Formatting Marks", "View", Some("Ctrl+*"), None));

    // ----- Review / Proofing -----
    cmds.push(cmd("review.spelling", "Spelling & Grammar", "Review", Some("F7"), None));
    cmds.push(cmd("review.word_count", "Word Count", "Review", None, None));
    cmds.push(cmd("review.track_changes", "Track Changes", "Review", Some("Ctrl+Shift+E"), None));
    cmds.push(cmd("review.accept_all", "Accept All Changes", "Review", None, None));
    cmds.push(cmd("review.reject_all", "Reject All Changes", "Review", None, None));
    cmds.push(cmd("review.new_comment", "New Comment", "Review", Some("Ctrl+Alt+M"), None));
    cmds.push(cmd("review.delete_all_comments", "Delete All Comments", "Review", None, None));

    // ----- Tools / Settings -----
    cmds.push(cmd("tools.options", "Options / Preferences", "Tools", Some("Ctrl+,"), Some("Open application settings")));
    cmds.push(cmd("tools.macros", "Record Macro...", "Tools", None, None));
    cmds.push(cmd("tools.word_count", "Word Count Details", "Tools", None, None));
    cmds.push(cmd("tools.language", "Set Language...", "Tools", None, Some("Set the proofing language")));

    cmds
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cmd(
    id: &str,
    label: &str,
    category: &str,
    shortcut: Option<&str>,
    description: Option<&str>,
) -> PaletteCommand {
    PaletteCommand {
        id: id.to_string(),
        label: label.to_string(),
        category: category.to_string(),
        shortcut: shortcut.map(|s| s.to_string()),
        description: description.map(|s| s.to_string()),
        enabled: true,
        use_count: 0,
    }
}
