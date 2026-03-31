use docx_rs::*;
use crate::ir;

/// Parse a DOCX file (bytes) into a Document IR.
/// Uses docx-rs reader + serialization workarounds for private fields.
pub fn read_docx(bytes: &[u8]) -> Result<ir::Document, String> {
    let docx = docx_rs::read_docx(bytes)
        .map_err(|e| format!("failed to read docx: {e:?}"))?;

    let mut doc = ir::Document::new();
    let mut raw_blocks = Vec::new();

    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
                convert_paragraph(para, &mut raw_blocks);
            }
            DocumentChild::Table(table) => {
                raw_blocks.push(RawBlock::Block(convert_table(table)));
            }
            _ => {}
        }
    }

    // Post-process: merge consecutive list items and code lines
    doc.children = merge_consecutive(raw_blocks);

    Ok(doc)
}

/// Detect if a run uses a monospace/code font by serializing RunFonts to JSON.
fn is_code_font(fonts: &Option<RunFonts>) -> bool {
    if let Some(f) = fonts {
        let json = serde_json::to_string(f).unwrap_or_default();
        json.contains("Courier") || json.contains("Consolas") || json.contains("Mono")
    } else {
        false
    }
}

/// Extract font name from RunFonts via serialization.
fn extract_font_name(fonts: &Option<RunFonts>) -> Option<String> {
    if let Some(f) = fonts {
        let json = serde_json::to_string(f).unwrap_or_default();
        // Parse {"ascii":"FontName",...}
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(name) = v.get("ascii").and_then(|v| v.as_str()) {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Tag used internally to identify code/blockquote paragraphs before merging.
#[derive(Debug)]
enum RawBlock {
    CodeLine(String),
    Block(ir::Block),
}

fn convert_paragraph(para: &Paragraph, blocks: &mut Vec<RawBlock>) {
    let mut runs = Vec::new();
    let mut is_heading = false;
    let mut heading_level = 0u8;
    let mut alignment = ir::Alignment::Left;
    let mut has_page_break = false;
    let mut is_indented = false;
    let mut all_code_font = true;
    let mut has_any_text = false;

    // Check paragraph style
    if let Some(ref style) = para.property.style {
        let id = &style.val;
        if id.contains("Heading1") || id == "1" { is_heading = true; heading_level = 1; }
        else if id.contains("Heading2") || id == "2" { is_heading = true; heading_level = 2; }
        else if id.contains("Heading3") || id == "3" { is_heading = true; heading_level = 3; }
        else if id.contains("Heading4") || id == "4" { is_heading = true; heading_level = 4; }
        else if id.contains("Heading5") || id == "5" { is_heading = true; heading_level = 5; }
        else if id.contains("Heading6") || id == "6" { is_heading = true; heading_level = 6; }
    }

    // Check alignment
    if let Some(ref jc) = para.property.alignment {
        alignment = match jc.val.as_str() {
            "center" => ir::Alignment::Center,
            "right" => ir::Alignment::Right,
            "both" | "justify" => ir::Alignment::Justify,
            _ => ir::Alignment::Left,
        };
    }

    // Check page break before
    if let Some(true) = para.property.page_break_before {
        has_page_break = true;
    }

    // Check indentation (blockquote detection)
    if let Some(ref indent) = para.property.indent {
        if let Some(start) = indent.start {
            if start >= 360 {
                is_indented = true;
            }
        }
    }

    // Convert runs and detect code font
    for child in &para.children {
        if let ParagraphChild::Run(run) = child {
            let ir_run = convert_run(run);
            if !ir_run.text.trim().is_empty() {
                has_any_text = true;
                if !is_code_font(&run.run_property.fonts) {
                    all_code_font = false;
                }
            }
            runs.push(ir_run);
        }
    }

    // Insert page break before this block
    if has_page_break {
        blocks.push(RawBlock::Block(ir::Block::PageBreak));
    }

    // Code font paragraphs → code lines (will be merged later)
    if all_code_font && has_any_text && !is_heading && !is_indented {
        let text: String = runs.iter().map(|r| r.text.as_str()).collect();
        blocks.push(RawBlock::CodeLine(text));
        return;
    }

    let props = ir::ParaProperties {
        alignment,
        ..Default::default()
    };

    // List items
    let is_list_item = para.property.numbering_property.is_some();

    if is_heading {
        for run in &mut runs {
            if heading_level <= 2 {
                run.properties.bold = false;
            }
        }
        blocks.push(RawBlock::Block(ir::Block::Heading(ir::Heading {
            level: heading_level,
            runs,
            properties: props,
        })));
    } else if is_list_item {
        blocks.push(RawBlock::Block(ir::Block::List(ir::List {
            ordered: false,
            items: vec![ir::ListItem { runs, children: Vec::new(), checked: None }],
        })));
    } else if is_indented && !is_heading {
        // Indented paragraph → blockquote
        blocks.push(RawBlock::Block(ir::Block::BlockQuote(vec![
            ir::Block::Paragraph(ir::Paragraph { runs, properties: props })
        ])));
    } else {
        blocks.push(RawBlock::Block(ir::Block::Paragraph(ir::Paragraph { runs, properties: props })));
    }
}

fn convert_run(run: &docx_rs::Run) -> ir::Run {
    let mut text = String::new();
    let mut props = ir::RunProperties::default();

    if run.run_property.bold.is_some() { props.bold = true; }
    if run.run_property.italic.is_some() { props.italic = true; }
    if run.run_property.underline.is_some() { props.underline = true; }
    if run.run_property.strike.is_some() { props.strikethrough = true; }

    // Extract text color (val is private, use serde to access)
    if let Some(ref color) = run.run_property.color {
        if let Ok(val) = serde_json::to_value(color) {
            if let Some(s) = val.as_str() {
                props.color = Some(s.to_string());
            }
        }
    }

    // Extract highlight color
    if let Some(ref highlight) = run.run_property.highlight {
        if let Ok(val) = serde_json::to_value(highlight) {
            if let Some(s) = val.as_str() {
                props.highlight = Some(s.to_string());
            }
        }
    }

    // Extract font name
    if let Some(name) = extract_font_name(&run.run_property.fonts) {
        props.font = Some(name);
    }

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
                    let mut sub_blocks = Vec::new();
                    convert_paragraph(para, &mut sub_blocks);
                    for rb in sub_blocks {
                        match rb {
                            RawBlock::Block(ir::Block::Paragraph(p)) => cell_runs.extend(p.runs),
                            RawBlock::Block(other) => cell_blocks.push(other),
                            RawBlock::CodeLine(text) => {
                                cell_runs.push(ir::Run::text(text).with_code());
                            }
                        }
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

/// Merge consecutive RawBlocks: code lines → code block, list items → list.
fn merge_consecutive(raw: Vec<RawBlock>) -> Vec<ir::Block> {
    let mut result: Vec<ir::Block> = Vec::new();
    let mut code_lines: Vec<String> = Vec::new();

    for rb in raw {
        match rb {
            RawBlock::CodeLine(line) => {
                code_lines.push(line);
            }
            RawBlock::Block(block) => {
                // Flush accumulated code lines
                if !code_lines.is_empty() {
                    result.push(ir::Block::CodeBlock(ir::CodeBlock {
                        language: String::new(),
                        content: code_lines.join("\n") + "\n",
                    }));
                    code_lines.clear();
                }

                // Merge consecutive single-item lists
                if let ir::Block::List(list) = &block {
                    if let Some(ir::Block::List(prev)) = result.last_mut() {
                        if prev.ordered == list.ordered {
                            prev.items.extend(list.items.clone());
                            continue;
                        }
                    }
                }

                // Merge consecutive blockquotes
                if let ir::Block::BlockQuote(inner) = &block {
                    if let Some(ir::Block::BlockQuote(prev)) = result.last_mut() {
                        prev.extend(inner.clone());
                        continue;
                    }
                }

                result.push(block);
            }
        }
    }

    // Flush remaining code lines
    if !code_lines.is_empty() {
        result.push(ir::Block::CodeBlock(ir::CodeBlock {
            language: String::new(),
            content: code_lines.join("\n") + "\n",
        }));
    }

    result
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

    fn roundtrip_md(md: &str) -> String {
        let doc = roundtrip(md);
        crate::render_markdown::render_to_markdown(&doc)
    }

    #[test]
    fn rt_heading() {
        let md = roundtrip_md("# Test Heading");
        assert!(md.contains("# "), "should have heading marker");
        assert!(md.contains("Test Heading"));
    }

    #[test]
    fn rt_bold() {
        let md = roundtrip_md("**bold text**");
        assert!(md.contains("**bold"), "should have bold markers");
    }

    #[test]
    fn rt_italic() {
        let md = roundtrip_md("*italic text*");
        assert!(md.contains("*italic"), "should have italic markers");
    }

    #[test]
    fn rt_centered() {
        let md = roundtrip_md("{center}# Centered Title");
        assert!(md.contains("{center}"), "should have center prefix");
        assert!(md.contains("Centered Title"));
    }

    #[test]
    fn rt_table() {
        let md = roundtrip_md("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(md.contains("|"), "should have table pipes");
        assert!(md.contains("A"));
        assert!(md.contains("1"));
    }

    #[test]
    fn rt_list() {
        let md = roundtrip_md("- First\n- Second\n- Third");
        assert!(md.contains("- "), "should have list markers: {md}");
        assert!(md.contains("First"));
        assert!(md.contains("Second"));
    }

    #[test]
    fn rt_page_break() {
        let md = roundtrip_md("Before\n\n{pagebreak}\n\nAfter");
        assert!(md.contains("{pagebreak}"), "should have page break marker: {md}");
        assert!(md.contains("Before"));
        assert!(md.contains("After"));
    }

    #[test]
    fn rt_code_block() {
        let md = roundtrip_md("```rust\nfn main() {}\n```");
        assert!(md.contains("```"), "should have code fences: {md}");
        assert!(md.contains("fn main()"));
    }

    #[test]
    fn rt_blockquote() {
        let md = roundtrip_md("> Quoted text here");
        assert!(md.contains(">"), "should have blockquote marker: {md}");
        assert!(md.contains("Quoted text"));
    }

    #[test]
    fn rt_valid_docx() {
        let doc = parse_markdown("# Hello\n\nWorld");
        let style = DocStyle::default();
        let bytes = render_to_docx(&doc, &style).unwrap();
        let result = read_docx(&bytes);
        assert!(result.is_ok());
    }
}
