//! # rw-mail-merge
//!
//! Mail merge functionality for Rust Writer.
//!
//! - Data source connection (CSV, JSON, spreadsheets)
//! - Field mapping (data columns to merge fields)
//! - Preview merged results
//! - Output to individual documents, printer, or email
//! - Rules (conditional fields: IF, SKIP, NEXT)
//! - Address block and greeting line helpers

use std::collections::HashMap;
use rw_document::{Document, Block, Inline, TextRun};
use rw_document::inline::FieldType;

/// A data source for mail merge.
#[derive(Debug, Clone)]
pub struct DataSource {
    /// Column names
    pub columns: Vec<String>,
    /// Rows of data (each row is a map of column name to value)
    pub records: Vec<HashMap<String, String>>,
}

/// Data source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSourceType {
    Csv,
    Json,
    Manual,
}

/// Mail merge output target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeOutput {
    /// Create individual documents
    NewDocuments,
    /// Send to printer
    Printer,
    /// Send as email
    Email,
}

/// Mail merge configuration.
#[derive(Debug, Clone)]
pub struct MergeConfig {
    pub source_type: DataSourceType,
    pub output: MergeOutput,
    /// Which records to merge (None = all)
    pub record_range: Option<(usize, usize)>,
}

/// A mail merge field placeholder.
#[derive(Debug, Clone)]
pub struct MergeField {
    pub name: String,
}

/// The mail merge engine.
#[derive(Debug, Default)]
pub struct MailMerge {
    records: Vec<HashMap<String, String>>,
}

impl MailMerge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data records to use for merging.
    pub fn set_data_source(&mut self, records: Vec<HashMap<String, String>>) {
        self.records = records;
    }

    /// Produce one merged `Document` per record by substituting merge fields.
    pub fn merge_document(&self, template: &Document) -> Vec<Document> {
        self.records
            .iter()
            .map(|record| self.merge_one(template, record))
            .collect()
    }

    fn merge_one(&self, template: &Document, record: &HashMap<String, String>) -> Document {
        let mut doc = template.clone();

        for section in &mut doc.sections {
            for block in &mut section.content {
                merge_block(block, record);
            }
        }

        doc
    }
}

/// Recursively walk a block and replace MergeField inlines with their values.
fn merge_block(block: &mut Block, record: &HashMap<String, String>) {
    match block {
        Block::Paragraph(para) => {
            let mut new_content = Vec::with_capacity(para.content.len());
            for inline in para.content.drain(..) {
                match inline {
                    Inline::Field(mut field_ref) => {
                        if let FieldType::MergeField { ref field_name } = field_ref.field_type {
                            let value = record
                                .get(field_name)
                                .cloned()
                                .unwrap_or_else(|| format!("«{}»", field_name));
                            // Replace the field with a text run carrying the merged value
                            let mut run = TextRun::new(value);
                            run.properties = field_ref.properties.clone();
                            new_content.push(Inline::Text(run));
                        } else {
                            new_content.push(Inline::Field(field_ref));
                        }
                    }
                    other => new_content.push(other),
                }
            }
            para.content = new_content;
        }
        Block::Table(tbl) => {
            for cell in &mut tbl.cells {
                for b in &mut cell.content {
                    merge_block(b, record);
                }
            }
        }
        Block::Frame(frame) => {
            for b in &mut frame.content {
                merge_block(b, record);
            }
        }
        Block::SubSection(sub) => {
            for b in &mut sub.content {
                merge_block(b, record);
            }
        }
        _ => {}
    }
}
