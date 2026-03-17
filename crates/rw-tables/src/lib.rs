//! # rw-tables
//!
//! Table engine for Rust Writer.
//!
//! Provides high-level operations for working with tables:
//! - Table creation (from dimensions, from text, from data)
//! - Row/column insertion and deletion
//! - Cell merging and splitting
//! - Cell formatting (borders, shading, alignment)
//! - Table autoformat / style application
//! - Sorting (ascending/descending, by column, multiple keys)
//! - Simple cell formulas (SUM, AVERAGE, COUNT, MIN, MAX)
//! - Table-to-text and text-to-table conversion
//! - Auto-fit column widths

pub mod autoformat;
pub mod formulas;
pub mod operations;
pub mod sorting;

/// A table cell address (row, column), 0-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellAddress {
    pub row: usize,
    pub col: usize,
}

impl CellAddress {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// A range of cells in a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellAddress,
    pub end: CellAddress,
}

impl CellRange {
    pub fn new(start: CellAddress, end: CellAddress) -> Self {
        Self { start, end }
    }

    /// Iterate over all cell addresses in this range.
    pub fn iter(&self) -> impl Iterator<Item = CellAddress> {
        let start = self.start;
        let end = self.end;
        (start.row..=end.row)
            .flat_map(move |row| (start.col..=end.col).map(move |col| CellAddress::new(row, col)))
    }
}
