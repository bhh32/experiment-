//! Table sorting operations.

use rw_document::block::TableBlock;

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

/// Sort the table rows according to the provided sort keys.
/// Each `SortKey` has `column`, `ascending` (via SortDirection), and `sort_type`.
pub fn sort_table(table: &mut TableBlock, keys: &[SortKey]) {
    if keys.is_empty() {
        return;
    }

    let cols = table.cols as usize;
    let rows = table.rows as usize;

    if rows == 0 || cols == 0 {
        return;
    }

    // Split cells into rows (Vec<Vec<TableCell>>)
    let mut row_data: Vec<Vec<rw_document::block::TableCell>> = table
        .cells
        .chunks(cols)
        .map(|chunk| chunk.to_vec())
        .collect();

    // Sort the rows using the provided keys
    row_data.sort_by(|a, b| {
        for key in keys {
            let col = key.column;
            if col >= cols {
                continue;
            }
            let text_a = cell_text(&a[col]);
            let text_b = cell_text(&b[col]);

            let ord = match key.sort_type {
                SortType::Number => {
                    let na = text_a.trim().parse::<f64>().unwrap_or(f64::NAN);
                    let nb = text_b.trim().parse::<f64>().unwrap_or(f64::NAN);
                    na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortType::Date => {
                    // Simple lexicographic date comparison (works for ISO 8601)
                    text_a.trim().cmp(text_b.trim())
                }
                SortType::Text => {
                    if key.case_sensitive {
                        text_a.trim().cmp(text_b.trim())
                    } else {
                        text_a.trim().to_lowercase().cmp(&text_b.trim().to_lowercase())
                    }
                }
            };

            let ord = if key.direction == SortDirection::Descending {
                ord.reverse()
            } else {
                ord
            };

            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });

    // Flatten back
    table.cells = row_data.into_iter().flatten().collect();
}

/// Extract plain text from the first paragraph in a cell.
fn cell_text(cell: &rw_document::block::TableCell) -> String {
    for block in &cell.content {
        if let rw_document::block::Block::Paragraph(para) = block {
            return para.plain_text();
        }
    }
    String::new()
}
