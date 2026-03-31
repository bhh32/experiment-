use docx_rs::*;
use crate::ir;

/// Parse a DOCX file (bytes) into a Document IR.
/// Note: docx-rs 0.4's reader has many private fields, so we extract
/// what we can and gracefully skip what's inaccessible.
pub fn read_docx(bytes: &[u8]) -> Result<ir::Document, String> {
    let docx = docx_rs::read_docx(bytes)
        .map_err(|e| format!("failed to read docx: {e:?}"))?;

    let mut doc = ir::Document::new();

    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
                let block = convert_paragraph(para);
                doc.children.push(block);
            }
            DocumentChild::Table(table) => {
                let block = convert_table(table);
                doc.children.push(block);
            }
            _ => {}
        }
    }

    // Post-process: merge consecutive single-item lists into multi-item lists
    doc.children = merge_consecutive_lists(doc.children);

    Ok(doc)
}

/// Merge consecutive single-item List blocks into multi-item lists.
fn merge_consecutive_lists(blocks: Vec<ir::Block>) -> Vec<ir::Block> {
    let mut result: Vec<ir::Block> = Vec::new();

    for block in blocks {
        if let ir::Block::List(list) = &block {
            if let Some(ir::Block::List(prev_list)) = result.last_mut() {
                // Merge into previous list if same type
                if prev_list.ordered == list.ordered {
                    prev_list.items.extend(list.items.clone());
                    continue;
                }
            }
        }
        result.push(block);
    }

    result
}

fn convert_paragraph(para: &docx_rs::Paragraph) -> ir::Block {
    let mut runs = Vec::new();
    let mut is_heading = false;
    let mut heading_level = 0u8;
    let mut alignment = ir::Alignment::Left;

    // Check paragraph style for headings
    if let Some(ref style) = para.property.style {
        let id = &style.val;
        if id.contains("Heading1") || id == "1" {
            is_heading = true;
            heading_level = 1;
        } else if id.contains("Heading2") || id == "2" {
            is_heading = true;
            heading_level = 2;
        } else if id.contains("Heading3") || id == "3" {
            is_heading = true;
            heading_level = 3;
        } else if id.contains("Heading4") || id == "4" {
            is_heading = true;
            heading_level = 4;
        } else if id.contains("Heading5") || id == "5" {
            is_heading = true;
            heading_level = 5;
        } else if id.contains("Heading6") || id == "6" {
            is_heading = true;
            heading_level = 6;
        }
    }

    // Check alignment — jc.val is a String in docx-rs reader
    if let Some(ref jc) = para.property.alignment {
        alignment = match jc.val.as_str() {
            "center" => ir::Alignment::Center,
            "right" => ir::Alignment::Right,
            "both" | "justify" => ir::Alignment::Justify,
            _ => ir::Alignment::Left,
        };
    }

    // Check for list numbering
    let is_list_item = para.property.numbering_property.is_some();

    // Convert runs
    for child in &para.children {
        if let ParagraphChild::Run(run) = child {
            let ir_run = convert_run(run);
            runs.push(ir_run);
        }
    }

    // If this is a list item, wrap it in a single-item list
    // The caller will need to merge consecutive list items
    if is_list_item && !is_heading {
        return ir::Block::List(ir::List {
            ordered: false, // Can't reliably detect ordered vs bullet from docx-rs reader
            items: vec![ir::ListItem {
                runs,
                children: Vec::new(),
                checked: None,
            }],
        });
    }

    let props = ir::ParaProperties {
        alignment,
        ..Default::default()
    };

    if is_heading {
        // Strip bold from heading runs — headings are inherently bold,
        // so bold in the DOCX is just the heading style, not user-applied bold
        for run in &mut runs {
            if heading_level <= 2 {
                run.properties.bold = false;
            }
        }
        ir::Block::Heading(ir::Heading {
            level: heading_level,
            runs,
            properties: props,
        })
    } else {
        ir::Block::Paragraph(ir::Paragraph { runs, properties: props })
    }
}

fn convert_run(run: &docx_rs::Run) -> ir::Run {
    let mut text = String::new();
    let mut props = ir::RunProperties::default();

    // Extract properties that are publicly accessible
    if run.run_property.bold.is_some() {
        props.bold = true;
    }
    if run.run_property.italic.is_some() {
        props.italic = true;
    }
    if run.run_property.underline.is_some() {
        props.underline = true;
    }
    if run.run_property.strike.is_some() {
        props.strikethrough = true;
    }

    // Extract text content
    for child in &run.children {
        if let RunChild::Text(t) = child {
            text.push_str(&t.text);
        }
    }

    ir::Run { text, properties: props }
}

fn convert_table(table: &docx_rs::Table) -> ir::Block {
    let mut header = Vec::new();
    let mut rows = Vec::new();
    let mut is_first_row = true;

    for row in &table.rows {
        let TableChild::TableRow(tr) = row;

        let mut cells = Vec::new();
        for cell in &tr.cells {
            let TableRowChild::TableCell(tc) = cell;

            let mut cell_runs = Vec::new();
            let mut cell_blocks = Vec::new();

            for tc_child in &tc.children {
                if let TableCellContent::Paragraph(para) = tc_child {
                    let block = convert_paragraph(para);
                    match block {
                        ir::Block::Paragraph(p) => cell_runs.extend(p.runs),
                        other => cell_blocks.push(other),
                    }
                }
            }

            cells.push(ir::TableCell {
                runs: cell_runs,
                blocks: cell_blocks,
                alignment: ir::Alignment::Left,
                is_header: is_first_row,
                col_span: 1,
                row_span: 1,
                shading: None,
            });
        }

        if is_first_row {
            header = cells;
            is_first_row = false;
        } else {
            rows.push(cells);
        }
    }

    ir::Block::Table(ir::Table {
        header,
        rows,
        properties: ir::TableProperties::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_docx::render_to_docx;
    use crate::parse::parse_markdown;
    use shared::DocStyle;

    fn roundtrip(md: &str) -> ir::Document {
        let doc = parse_markdown(md);
        let style = DocStyle::default();
        let bytes = render_to_docx(&doc, &style).unwrap();
        read_docx(&bytes).unwrap()
    }

    #[test]
    fn roundtrip_heading() {
        let doc = roundtrip("# Test Heading");
        let headings: Vec<_> = doc.children.iter().filter_map(|b| {
            if let ir::Block::Heading(h) = b { Some(h) } else { None }
        }).collect();
        assert!(!headings.is_empty(), "should have at least one heading");
        assert_eq!(headings[0].level, 1);
        let text: String = headings[0].runs.iter().map(|r| r.text.as_str()).collect();
        assert!(text.contains("Test Heading"), "heading text: {text}");
    }

    #[test]
    fn roundtrip_paragraph() {
        let doc = roundtrip("A simple paragraph.");
        let paras: Vec<_> = doc.children.iter().filter_map(|b| {
            if let ir::Block::Paragraph(p) = b { Some(p) } else { None }
        }).collect();
        assert!(!paras.is_empty());
        let text: String = paras.iter()
            .flat_map(|p| p.runs.iter())
            .map(|r| r.text.as_str())
            .collect();
        assert!(text.contains("A simple paragraph"), "text: {text}");
    }

    #[test]
    fn roundtrip_bold() {
        let doc = roundtrip("**bold text**");
        let has_bold = doc.children.iter().any(|b| {
            if let ir::Block::Paragraph(p) = b {
                p.runs.iter().any(|r| r.properties.bold && r.text.contains("bold"))
            } else {
                false
            }
        });
        assert!(has_bold, "should have bold run");
    }

    #[test]
    fn roundtrip_italic() {
        let doc = roundtrip("*italic text*");
        let has_italic = doc.children.iter().any(|b| {
            if let ir::Block::Paragraph(p) = b {
                p.runs.iter().any(|r| r.properties.italic && r.text.contains("italic"))
            } else {
                false
            }
        });
        assert!(has_italic, "should have italic run");
    }

    #[test]
    fn roundtrip_table() {
        let doc = roundtrip("| A | B |\n|---|---|\n| 1 | 2 |");
        let tables: Vec<_> = doc.children.iter().filter_map(|b| {
            if let ir::Block::Table(t) = b { Some(t) } else { None }
        }).collect();
        assert!(!tables.is_empty(), "should have a table");
        assert!(!tables[0].header.is_empty());
    }

    #[test]
    fn roundtrip_centered() {
        let doc = roundtrip("{center}# Centered Title");
        let headings: Vec<_> = doc.children.iter().filter_map(|b| {
            if let ir::Block::Heading(h) = b { Some(h) } else { None }
        }).collect();
        assert!(!headings.is_empty());
        assert_eq!(headings[0].properties.alignment, ir::Alignment::Center);
    }

    #[test]
    fn reads_valid_docx() {
        let doc = parse_markdown("# Hello\n\nWorld");
        let style = DocStyle::default();
        let bytes = render_to_docx(&doc, &style).unwrap();
        let result = read_docx(&bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().schema_version, ir::SCHEMA_VERSION);
    }
}
