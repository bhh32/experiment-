//! Table sorting operations.

/// Sort configuration for a table.
#[derive(Debug, Clone)]
pub struct SortConfig {
    /// Sort keys in priority order
    pub keys: Vec<SortKey>,
    /// Whether the first row is a header (excluded from sorting)
    pub has_header: bool,
}

#[derive(Debug, Clone)]
pub struct SortKey {
    /// Column index to sort by
    pub column: usize,
    /// Sort direction
    pub direction: SortDirection,
    /// Sort type (how to compare values)
    pub sort_type: SortType,
    /// Case sensitive comparison
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortType {
    /// Sort as text (lexicographic)
    Text,
    /// Sort as numbers
    Number,
    /// Sort as dates
    Date,
}
