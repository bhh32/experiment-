//! The `data` command topic for oxy-sf: SOQL query.

#![forbid(unsafe_code)]

use clap::{Args, Subcommand};
use oxysf_core::api::rest::{self, QueryResult};
use oxysf_core::context::CommandContext;
use oxysf_core::error::{Result, SfError};
use oxysf_core::output::{print_success, render_table, OutputMode};

/// The `data` topic command tree.
#[derive(Debug, Subcommand)]
pub enum DataCommand {
    /// Execute a SOQL query against the target org.
    Query(QueryArgs),
}

/// Arguments for `data query`.
#[derive(Debug, Args)]
pub struct QueryArgs {
    /// The SOQL query to execute.
    #[arg(short = 'q', long)]
    pub query: String,
    /// Output format for the records: human, json, or csv.
    #[arg(long, default_value = "human")]
    pub result_format: String,
}

/// Dispatch a `data` subcommand.
pub fn run(ctx: &CommandContext, cmd: &DataCommand) -> Result<()> {
    match cmd {
        DataCommand::Query(args) => query(ctx, args),
    }
}

fn query(ctx: &CommandContext, args: &QueryArgs) -> Result<()> {
    let mut conn = ctx.connection()?;
    let result = rest::query(&mut conn, &args.query)?;

    // `--json` global flag forces JSON regardless of --result-format.
    if ctx.output == OutputMode::Json || args.result_format.eq_ignore_ascii_case("json") {
        print_success(OutputMode::Json, result, vec![], |_| {});
        return Ok(());
    }

    match args.result_format.to_ascii_lowercase().as_str() {
        "human" => {
            print_success(OutputMode::Human, result, vec![], render_human);
            Ok(())
        }
        "csv" => print_csv(&result),
        other => Err(SfError::Config(format!(
            "unknown result format '{other}' (expected human, json, or csv)"
        ))),
    }
}

/// Collect ordered, de-duplicated column names from the records (excluding the
/// `attributes` metadata object Salesforce attaches to each record).
fn columns(result: &QueryResult) -> Vec<String> {
    let mut cols: Vec<String> = Vec::new();
    for rec in &result.records {
        if let Some(obj) = rec.as_object() {
            for key in obj.keys() {
                if key == "attributes" {
                    continue;
                }
                if !cols.iter().any(|c| c == key) {
                    cols.push(key.clone());
                }
            }
        }
    }
    cols
}

/// Render a single JSON value as a flat cell string.
fn cell(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
    }
}

fn render_human(result: &QueryResult) {
    let cols = columns(result);
    if cols.is_empty() {
        println!("No records found. (Total: {})", result.total_size);
        return;
    }
    let headers: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
    let rows: Vec<Vec<String>> = result
        .records
        .iter()
        .map(|rec| cols.iter().map(|c| cell(rec.get(c))).collect())
        .collect();
    let table = render_table(&headers, &rows);
    println!("{table}");
    println!("Total records: {}", result.total_size);
}

fn print_csv(result: &QueryResult) -> Result<()> {
    let cols = columns(result);
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(&cols)
        .map_err(|e| SfError::Io(std::io::Error::other(e.to_string())))?;
    for rec in &result.records {
        let row: Vec<String> = cols.iter().map(|c| cell(rec.get(c))).collect();
        wtr.write_record(&row)
            .map_err(|e| SfError::Io(std::io::Error::other(e.to_string())))?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> QueryResult {
        QueryResult {
            total_size: 2,
            records: vec![
                serde_json::json!({"attributes": {"type": "Account"}, "Id": "1", "Name": "Acme"}),
                serde_json::json!({"attributes": {"type": "Account"}, "Id": "2", "Name": "Globex"}),
            ],
        }
    }

    #[test]
    fn columns_skip_attributes() {
        let cols = columns(&sample());
        assert_eq!(cols, vec!["Id".to_string(), "Name".to_string()]);
    }

    #[test]
    fn cell_formats_values() {
        assert_eq!(cell(Some(&serde_json::json!("x"))), "x");
        assert_eq!(cell(Some(&serde_json::json!(5))), "5");
        assert_eq!(cell(Some(&serde_json::Value::Null)), "");
        assert_eq!(cell(None), "");
    }
}
