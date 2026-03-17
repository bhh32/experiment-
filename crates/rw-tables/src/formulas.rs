//! Simple table cell formulas.
//!
//! Supports basic formulas like =SUM(A1:A5), =AVERAGE(B2:B10), etc.
//! Uses Word-compatible cell references (A1 notation where columns
//! are letters and rows are numbers).

use rw_document::block::TableBlock;
use thiserror::Error;

/// A table cell formula.
#[derive(Debug, Clone)]
pub enum CellFormula {
    Sum(String),     // range expression like "A1:A5"
    Average(String),
    Count(String),
    Min(String),
    Max(String),
    Product(String),
}

impl CellFormula {
    /// Parse a formula string like "=SUM(A1:A5)".
    pub fn parse(formula: &str) -> Option<Self> {
        let formula = formula.trim();
        if !formula.starts_with('=') {
            return None;
        }
        let formula = &formula[1..].trim();
        let upper = formula.to_uppercase();

        if let Some(range) = extract_function_arg(&upper, "SUM") {
            Some(CellFormula::Sum(range))
        } else if let Some(range) = extract_function_arg(&upper, "AVERAGE") {
            Some(CellFormula::Average(range))
        } else if let Some(range) = extract_function_arg(&upper, "COUNT") {
            Some(CellFormula::Count(range))
        } else if let Some(range) = extract_function_arg(&upper, "MIN") {
            Some(CellFormula::Min(range))
        } else if let Some(range) = extract_function_arg(&upper, "MAX") {
            Some(CellFormula::Max(range))
        } else if let Some(range) = extract_function_arg(&upper, "PRODUCT") {
            Some(CellFormula::Product(range))
        } else {
            None
        }
    }
}

fn extract_function_arg(formula: &str, func_name: &str) -> Option<String> {
    if formula.starts_with(func_name) {
        let rest = &formula[func_name.len()..];
        if rest.starts_with('(') && rest.ends_with(')') {
            Some(rest[1..rest.len() - 1].to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Error type for formula evaluation.
#[derive(Debug, Error)]
pub enum FormulaError {
    #[error("Invalid formula syntax: {0}")]
    ParseError(String),
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Invalid cell reference: {0}")]
    InvalidReference(String),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("No values in range")]
    EmptyRange,
}

/// Evaluate a formula string against a table and return the numeric result.
pub fn evaluate_formula(formula: &str, table: &TableBlock) -> Result<f64, FormulaError> {
    let formula = formula.trim();
    if !formula.starts_with('=') {
        return Err(FormulaError::ParseError("Formula must start with '='".to_string()));
    }
    let body = formula[1..].trim().to_uppercase();

    let func_name: &str;
    let arg: String;

    if let Some(a) = extract_function_arg(&body, "SUM") {
        func_name = "SUM";
        arg = a;
    } else if let Some(a) = extract_function_arg(&body, "AVERAGE") {
        func_name = "AVERAGE";
        arg = a;
    } else if let Some(a) = extract_function_arg(&body, "COUNT") {
        func_name = "COUNT";
        arg = a;
    } else if let Some(a) = extract_function_arg(&body, "MIN") {
        func_name = "MIN";
        arg = a;
    } else if let Some(a) = extract_function_arg(&body, "MAX") {
        func_name = "MAX";
        arg = a;
    } else if let Some(a) = extract_function_arg(&body, "PRODUCT") {
        func_name = "PRODUCT";
        arg = a;
    } else {
        return Err(FormulaError::UnknownFunction(body));
    }

    let values = resolve_range(table, &arg)?;

    if values.is_empty() {
        return Err(FormulaError::EmptyRange);
    }

    match func_name {
        "SUM" => Ok(values.iter().sum()),
        "AVERAGE" => Ok(values.iter().sum::<f64>() / values.len() as f64),
        "COUNT" => Ok(values.len() as f64),
        "MIN" => Ok(values.iter().cloned().fold(f64::INFINITY, f64::min)),
        "MAX" => Ok(values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
        "PRODUCT" => Ok(values.iter().product()),
        _ => Err(FormulaError::UnknownFunction(func_name.to_string())),
    }
}

/// Resolve a range expression to a list of numeric values from the table.
fn resolve_range(table: &TableBlock, range: &str) -> Result<Vec<f64>, FormulaError> {
    let range = range.trim();

    // Handle ABOVE / LEFT keywords
    if range == "ABOVE" {
        return Ok(collect_column_values(table, 0));
    }
    if range == "LEFT" {
        return Ok(collect_row_values(table, 0));
    }

    // Handle A1:B3 range or single cell like A1
    if let Some(colon) = range.find(':') {
        let start_ref = &range[..colon];
        let end_ref = &range[colon + 1..];
        let (start_row, start_col) = parse_cell_ref(start_ref)?;
        let (end_row, end_col) = parse_cell_ref(end_ref)?;

        let cols = table.cols as usize;
        let mut values = Vec::new();
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let idx = row * cols + col;
                if idx < table.cells.len() {
                    if let Some(v) = cell_numeric_value(&table.cells[idx]) {
                        values.push(v);
                    }
                }
            }
        }
        Ok(values)
    } else {
        // Single cell
        let (row, col) = parse_cell_ref(range)?;
        let cols = table.cols as usize;
        let idx = row * cols + col;
        if idx < table.cells.len() {
            if let Some(v) = cell_numeric_value(&table.cells[idx]) {
                Ok(vec![v])
            } else {
                Ok(vec![])
            }
        } else {
            Err(FormulaError::InvalidReference(range.to_string()))
        }
    }
}

/// Parse a cell reference like "A1" -> (row 0, col 0), "B2" -> (row 1, col 1).
fn parse_cell_ref(cell_ref: &str) -> Result<(usize, usize), FormulaError> {
    let cell_ref = cell_ref.trim();
    let col_str: String = cell_ref.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let row_str: String = cell_ref.chars().skip_while(|c| c.is_ascii_alphabetic()).collect();

    if col_str.is_empty() || row_str.is_empty() {
        return Err(FormulaError::InvalidReference(cell_ref.to_string()));
    }

    // Convert column letters to 0-based index: A=0, B=1, ..., Z=25, AA=26
    let col = col_str
        .chars()
        .fold(0usize, |acc, c| acc * 26 + (c as usize - 'A' as usize + 1))
        - 1;

    let row: usize = row_str
        .parse::<usize>()
        .map_err(|_| FormulaError::InvalidReference(cell_ref.to_string()))?
        .saturating_sub(1); // 1-based to 0-based

    Ok((row, col))
}

/// Extract a numeric value from a cell (from its text content).
fn cell_numeric_value(cell: &rw_document::block::TableCell) -> Option<f64> {
    for block in &cell.content {
        if let rw_document::block::Block::Paragraph(para) = block {
            let text = para.plain_text().trim().to_string();
            if let Ok(v) = text.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// Collect all numeric values from an entire column (for ABOVE).
fn collect_column_values(table: &TableBlock, col: usize) -> Vec<f64> {
    let cols = table.cols as usize;
    let rows = table.rows as usize;
    let mut values = Vec::new();
    for row in 0..rows {
        let idx = row * cols + col;
        if idx < table.cells.len() {
            if let Some(v) = cell_numeric_value(&table.cells[idx]) {
                values.push(v);
            }
        }
    }
    values
}

/// Collect all numeric values from an entire row (for LEFT).
fn collect_row_values(table: &TableBlock, row: usize) -> Vec<f64> {
    let cols = table.cols as usize;
    let mut values = Vec::new();
    for col in 0..cols {
        let idx = row * cols + col;
        if idx < table.cells.len() {
            if let Some(v) = cell_numeric_value(&table.cells[idx]) {
                values.push(v);
            }
        }
    }
    values
}
