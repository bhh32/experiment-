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
