//! Backstage view — the full-page overlay that appears when clicking "File".
//!
//! Contains: New, Open, Save, Save As, Print, Export, Info, Options.

/// Backstage view pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackstagePage {
    Info,
    New,
    Open,
    Save,
    SaveAs,
    Print,
    Export,
    Options,
}

/// A recent document entry.
#[derive(Debug, Clone)]
pub struct RecentDocument {
    pub name: String,
    pub path: String,
    pub last_opened: String,
    pub pinned: bool,
}

// ---------------------------------------------------------------------------
// BackstageView
// ---------------------------------------------------------------------------

/// The action triggered when a backstage item is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackstageAction {
    New,
    Open,
    Save,
    SaveAs,
    Print,
    Export,
    Info,
    Recent,
    Close,
}

/// A single item in the backstage navigation panel.
#[derive(Debug, Clone)]
pub struct BackstageItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon name (symbolic icon)
    pub icon: String,
    /// The action associated with this item
    pub action_type: BackstageAction,
    /// Whether a separator should be rendered above this item
    pub separator_above: bool,
}

/// The complete backstage view structure.
#[derive(Debug, Clone)]
pub struct BackstageView {
    /// All navigation items
    pub items: Vec<BackstageItem>,
    /// Currently active page
    pub active_page: BackstageAction,
}

impl BackstageView {
    /// Create a new, empty backstage view.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active_page: BackstageAction::Info,
        }
    }

    /// Navigate to a specific action page.
    pub fn navigate(&mut self, action: BackstageAction) {
        self.active_page = action;
    }
}

impl Default for BackstageView {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default backstage view with all standard File menu items.
pub fn default_backstage() -> BackstageView {
    BackstageView {
        active_page: BackstageAction::Info,
        items: vec![
            BackstageItem {
                id: "info".to_string(),
                label: "Info".to_string(),
                icon: "dialog-information-symbolic".to_string(),
                action_type: BackstageAction::Info,
                separator_above: false,
            },
            BackstageItem {
                id: "new".to_string(),
                label: "New".to_string(),
                icon: "document-new-symbolic".to_string(),
                action_type: BackstageAction::New,
                separator_above: false,
            },
            BackstageItem {
                id: "open".to_string(),
                label: "Open".to_string(),
                icon: "document-open-symbolic".to_string(),
                action_type: BackstageAction::Open,
                separator_above: false,
            },
            BackstageItem {
                id: "recent".to_string(),
                label: "Recent".to_string(),
                icon: "document-open-recent-symbolic".to_string(),
                action_type: BackstageAction::Recent,
                separator_above: false,
            },
            BackstageItem {
                id: "save".to_string(),
                label: "Save".to_string(),
                icon: "document-save-symbolic".to_string(),
                action_type: BackstageAction::Save,
                separator_above: true,
            },
            BackstageItem {
                id: "save_as".to_string(),
                label: "Save As".to_string(),
                icon: "document-save-as-symbolic".to_string(),
                action_type: BackstageAction::SaveAs,
                separator_above: false,
            },
            BackstageItem {
                id: "print".to_string(),
                label: "Print".to_string(),
                icon: "document-print-symbolic".to_string(),
                action_type: BackstageAction::Print,
                separator_above: true,
            },
            BackstageItem {
                id: "export".to_string(),
                label: "Export".to_string(),
                icon: "document-export-symbolic".to_string(),
                action_type: BackstageAction::Export,
                separator_above: false,
            },
            BackstageItem {
                id: "close".to_string(),
                label: "Close".to_string(),
                icon: "window-close-symbolic".to_string(),
                action_type: BackstageAction::Close,
                separator_above: true,
            },
        ],
    }
}
