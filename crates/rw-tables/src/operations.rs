//! Table operations — high-level actions on tables.

use rw_document::block::{TableBlock, TableCell, TableCellProperties};
use rw_document::ElementId;
use crate::{CellAddress, CellRange};

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

/// Create an empty cell.
fn empty_cell() -> TableCell {
    TableCell {
        id: ElementId::new(),
        content: Vec::new(),
        properties: TableCellProperties::default(),
        col_span: 1,
        row_span: 1,
    }
}

/// Insert an empty row at `at_row` (0-based index, row is inserted before that index).
pub fn insert_row(table: &mut TableBlock, at_row: usize) {
    let cols = table.cols as usize;
    let at_row = at_row.min(table.rows as usize);
    let flat_index = at_row * cols;
    let new_row: Vec<TableCell> = (0..cols).map(|_| empty_cell()).collect();
    // Insert the cells at the correct flat position
    let mut new_cells = Vec::with_capacity(table.cells.len() + cols);
    new_cells.extend(table.cells.drain(..flat_index));
    new_cells.extend(new_row);
    new_cells.extend(table.cells.drain(..));
    table.cells = new_cells;
    table.rows += 1;
}

/// Insert an empty column at `at_col` (0-based, column inserted before that index).
pub fn insert_column(table: &mut TableBlock, at_col: usize) {
    let rows = table.rows as usize;
    let cols = table.cols as usize;
    let at_col = at_col.min(cols);
    let new_col_count = cols + 1;
    let mut new_cells: Vec<TableCell> = Vec::with_capacity(rows * new_col_count);

    for row in 0..rows {
        for col in 0..new_col_count {
            if col == at_col {
                new_cells.push(empty_cell());
            } else {
                let old_col = if col < at_col { col } else { col - 1 };
                new_cells.push(table.cells[row * cols + old_col].clone());
            }
        }
    }
    table.cells = new_cells;
    table.cols += 1;
}

/// Delete row at `row` (0-based).
pub fn delete_row(table: &mut TableBlock, row: usize) {
    if row >= table.rows as usize {
        return;
    }
    let cols = table.cols as usize;
    let start = row * cols;
    let end = start + cols;
    table.cells.drain(start..end);
    table.rows -= 1;
}

/// Delete column at `col` (0-based).
pub fn delete_column(table: &mut TableBlock, col: usize) {
    if col >= table.cols as usize {
        return;
    }
    let rows = table.rows as usize;
    let cols = table.cols as usize;
    let new_cols = cols - 1;
    let mut new_cells: Vec<TableCell> = Vec::with_capacity(rows * new_cols);

    for row in 0..rows {
        for c in 0..cols {
            if c != col {
                new_cells.push(table.cells[row * cols + c].clone());
            }
        }
    }
    table.cells = new_cells;
    table.cols -= 1;
}

/// Merge cells in the given range into the top-left cell.
pub fn merge_cells(table: &mut TableBlock, range: CellRange) {
    let cols = table.cols as usize;
    let rows_span = (range.end.row - range.start.row + 1) as u32;
    let cols_span = (range.end.col - range.start.col + 1) as u32;

    // Collect content from all cells in range into the top-left cell
    let mut merged_content = Vec::new();
    for addr in range.iter() {
        let idx = addr.row * cols + addr.col;
        if idx < table.cells.len() {
            let cell_content = table.cells[idx].content.clone();
            merged_content.extend(cell_content);
        }
    }

    // Update top-left cell
    let tl_idx = range.start.row * cols + range.start.col;
    if tl_idx < table.cells.len() {
        table.cells[tl_idx].content = merged_content;
        table.cells[tl_idx].row_span = rows_span;
        table.cells[tl_idx].col_span = cols_span;
    }

    // Clear other cells in range (mark as empty with span 0)
    for addr in range.iter() {
        if addr == range.start {
            continue;
        }
        let idx = addr.row * cols + addr.col;
        if idx < table.cells.len() {
            table.cells[idx].content.clear();
            table.cells[idx].row_span = 0;
            table.cells[idx].col_span = 0;
        }
    }
}

/// Split a (potentially merged) cell at `addr` into `rows x cols` individual cells.
pub fn split_cell(table: &mut TableBlock, addr: CellAddress, rows: usize, cols: usize) {
    let table_cols = table.cols as usize;
    let idx = addr.row * table_cols + addr.col;
    if idx >= table.cells.len() {
        return;
    }

    // Reset the cell span
    table.cells[idx].row_span = 1;
    table.cells[idx].col_span = 1;

    // For a proper split we would need to insert rows/cols — here we simply
    // reset adjacent spanned cells back to normal single cells (best-effort).
    for dr in 0..rows {
        for dc in 0..cols {
            if dr == 0 && dc == 0 {
                continue;
            }
            let r = addr.row + dr;
            let c = addr.col + dc;
            if r < table.rows as usize && c < table_cols {
                let i = r * table_cols + c;
                if i < table.cells.len()
                    && table.cells[i].row_span == 0
                    && table.cells[i].col_span == 0
                {
                    table.cells[i].row_span = 1;
                    table.cells[i].col_span = 1;
                }
            }
        }
    }
}

/// Auto-fit column widths based on content length (heuristic: 100 twips per character, min 720).
pub fn auto_fit_columns(table: &mut TableBlock) {
    let rows = table.rows as usize;
    let cols = table.cols as usize;
    if cols == 0 {
        return;
    }

    let mut col_widths: Vec<usize> = vec![1; cols];

    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            if idx < table.cells.len() {
                let text_len: usize = table.cells[idx]
                    .content
                    .iter()
                    .filter_map(|b| {
                        if let rw_document::block::Block::Paragraph(p) = b {
                            Some(p.plain_text().len())
                        } else {
                            None
                        }
                    })
                    .sum();
                if text_len > col_widths[col] {
                    col_widths[col] = text_len;
                }
            }
        }
    }

    // Apply widths: 100 twips per character, minimum 720 twips (~0.5 inch)
    for (col, &char_count) in col_widths.iter().enumerate() {
        let width_twips = ((char_count * 100).max(720)) as i32;
        for row in 0..rows {
            let idx = row * cols + col;
            if idx < table.cells.len() {
                table.cells[idx].properties.width =
                    Some(rw_document::block::TableWidth::Fixed(rw_document::Twips(width_twips)));
            }
        }
    }
}
