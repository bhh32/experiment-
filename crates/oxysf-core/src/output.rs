//! Output rendering: human-readable tables and machine JSON envelopes.
//!
//! Conventions:
//! - The JSON result (on success or error) goes to **stdout**.
//! - Human-readable success output goes to **stdout**.
//! - Human-readable errors go to **stderr**.
//! - Logs / progress indicators go to **stderr** (handled elsewhere).

use comfy_table::{ContentArrangement, Table};
use serde::Serialize;

use crate::error::SfError;

/// How command output should be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-readable text / tables.
    Human,
    /// Machine-readable JSON envelopes.
    Json,
}

/// The success envelope, matching `sf`'s shape: `{status, result, warnings}`.
#[derive(Debug, Serialize)]
pub struct Envelope<T: Serialize> {
    /// Process status code (0 = success).
    pub status: i32,
    /// The command result payload.
    pub result: T,
    /// Non-fatal warnings emitted during the command.
    pub warnings: Vec<String>,
}

impl<T: Serialize> Envelope<T> {
    /// Build a success envelope (status 0).
    pub fn success(result: T, warnings: Vec<String>) -> Self {
        Envelope {
            status: 0,
            result,
            warnings,
        }
    }
}

/// Print a successful result.
///
/// In [`OutputMode::Json`] this writes the [`Envelope`] as JSON to stdout. In
/// [`OutputMode::Human`] mode it invokes `render_human` so callers control the
/// text layout.
pub fn print_success<T, F>(mode: OutputMode, result: T, warnings: Vec<String>, render_human: F)
where
    T: Serialize,
    F: FnOnce(&T),
{
    match mode {
        OutputMode::Json => {
            let env = Envelope::success(result, warnings);
            match serde_json::to_string_pretty(&env) {
                Ok(s) => println!("{s}"),
                Err(e) => eprintln!("failed to serialize output: {e}"),
            }
        }
        OutputMode::Human => {
            for w in &warnings {
                eprintln!("Warning: {w}");
            }
            render_human(&result);
        }
    }
}

/// Print an error.
///
/// In [`OutputMode::Json`] this writes an error envelope
/// `{status, name, message, ...}` to stdout. In human mode it writes a friendly
/// message to stderr.
pub fn print_error(mode: OutputMode, err: &SfError) {
    match mode {
        OutputMode::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert("status".into(), serde_json::json!(err.exit_code()));
            obj.insert("name".into(), serde_json::json!(err.name()));
            obj.insert("message".into(), serde_json::json!(err.to_string()));
            obj.insert("warnings".into(), serde_json::json!([]));
            if let SfError::Api { error_code, .. } = err {
                obj.insert("errorCode".into(), serde_json::json!(error_code));
            }
            match serde_json::to_string_pretty(&serde_json::Value::Object(obj)) {
                Ok(s) => println!("{s}"),
                Err(e) => eprintln!("failed to serialize error: {e}"),
            }
        }
        OutputMode::Human => {
            eprintln!("Error: {err}");
        }
    }
}

/// Render a table to a `String` using `comfy-table`.
pub fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if !headers.is_empty() {
        table.set_header(headers.iter().map(|h| h.to_string()));
    }
    for row in rows {
        table.add_row(row.clone());
    }
    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Demo {
        username: String,
        org_id: String,
    }

    #[test]
    fn success_envelope_shape() {
        let env = Envelope::success(
            Demo {
                username: "user@example.com".into(),
                org_id: "00D000000000000".into(),
            },
            vec![],
        );
        let v = serde_json::to_value(&env).unwrap();
        assert_eq!(v["status"], 0);
        assert_eq!(v["result"]["username"], "user@example.com");
        assert_eq!(v["result"]["org_id"], "00D000000000000");
        assert!(v["warnings"].is_array());
    }

    #[test]
    fn table_contains_headers_and_rows() {
        let t = render_table(
            &["A", "B"],
            &[vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
        );
        assert!(t.contains('A'));
        assert!(t.contains('4'));
    }
}
