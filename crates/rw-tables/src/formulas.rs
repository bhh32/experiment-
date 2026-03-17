//! Simple table cell formulas.
//!
//! Supports basic formulas like =SUM(A1:A5), =AVERAGE(B2:B10), etc.
//! Uses Word-compatible cell references (A1 notation where columns
//! are letters and rows are numbers).

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
