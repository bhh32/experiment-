//! Table autoformat — predefined table styles.

/// A predefined table autoformat.
#[derive(Debug, Clone)]
pub struct TableAutoFormat {
    pub name: String,
    pub description: String,
    /// Whether to apply font formatting
    pub apply_font: bool,
    /// Whether to apply border formatting
    pub apply_borders: bool,
    /// Whether to apply shading
    pub apply_shading: bool,
    /// Whether to apply alignment
    pub apply_alignment: bool,
    /// Whether the first row is special (header)
    pub header_row: bool,
    /// Whether the last row is special (total)
    pub footer_row: bool,
    /// Whether the first column is special
    pub first_column: bool,
    /// Whether the last column is special
    pub last_column: bool,
    /// Whether to use banded rows
    pub banded_rows: bool,
    /// Whether to use banded columns
    pub banded_columns: bool,
}
