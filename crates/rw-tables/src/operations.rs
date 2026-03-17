//! Table operations — high-level actions on tables.

use crate::CellAddress;

/// Operations that can be performed on a table.
#[derive(Debug, Clone)]
pub enum TableOperation {
    /// Insert rows at the given position
    InsertRows { at: usize, count: usize, position: InsertPosition },
    /// Insert columns at the given position
    InsertColumns { at: usize, count: usize, position: InsertPosition },
    /// Delete rows
    DeleteRows { from: usize, count: usize },
    /// Delete columns
    DeleteColumns { from: usize, count: usize },
    /// Merge the selected cells into one
    MergeCells { from: CellAddress, to: CellAddress },
    /// Split a merged cell back into individual cells
    SplitCell { cell: CellAddress, rows: usize, cols: usize },
    /// Auto-fit column widths to content
    AutoFitContent,
    /// Auto-fit to window width
    AutoFitWindow,
    /// Set fixed column width
    SetColumnWidth { col: usize, width_twips: i32 },
    /// Set row height
    SetRowHeight { row: usize, height_twips: i32 },
    /// Apply a table style
    ApplyStyle { style_name: String },
}

#[derive(Debug, Clone, Copy)]
pub enum InsertPosition {
    Before,
    After,
}
