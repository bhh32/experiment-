use crate::ConversionError;
use comrak::{Arena, Options, parse_document};
use comrak::nodes::{NodeValue, ListType};
use docx_rs::*;

use shared::DocStyle;

/// Type alias for backward compatibility
pub type DocxStyle = DocStyle;

fn gfm_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts
}

fn font(name: &str) -> RunFonts {
    RunFonts::new().ascii(name).hi_ansi(name).cs(name).east_asia(name)
}

/// Track inline formatting state while walking the AST.
#[derive(Clone, Default)]
struct InlineState {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    code: bool,
    link_url: Option<String>,
    superscript: bool,
}

/// Convert markdown to DOCX with default styling.
pub fn markdown_to_docx(markdown: &str) -> Result<Vec<u8>, ConversionError> {
    markdown_to_docx_styled(markdown, &DocxStyle::default())
}

/// Pre-process markdown to extract alignment prefixes.
/// Returns (cleaned markdown, map of paragraph text -> alignment).
fn preprocess_alignments(markdown: &str) -> (String, std::collections::HashMap<String, AlignmentType>) {
    let mut clean_lines = Vec::new();
    let mut alignments = std::collections::HashMap::new();

    for line in markdown.lines() {
        if let Some(rest) = line.strip_prefix("{center}") {
            let key = rest.trim().to_string();
            if !key.is_empty() {
                alignments.insert(key, AlignmentType::Center);
            }
            clean_lines.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("{right}") {
            let key = rest.trim().to_string();
            if !key.is_empty() {
                alignments.insert(key, AlignmentType::Right);
            }
            clean_lines.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("{justify}") {
            let key = rest.trim().to_string();
            if !key.is_empty() {
                alignments.insert(key, AlignmentType::Justified);
            }
            clean_lines.push(rest.to_string());
        } else {
            clean_lines.push(line.to_string());
        }
    }

    (clean_lines.join("\n"), alignments)
}

/// Convert markdown to a fully-formatted DOCX byte buffer with custom styling.
pub fn markdown_to_docx_styled(markdown: &str, style: &DocxStyle) -> Result<Vec<u8>, ConversionError> {
    let (clean_md, alignments) = preprocess_alignments(markdown);
    let arena = Arena::new();
    let root = parse_document(&arena, &clean_md, &gfm_options());

    let mut doc = Docx::new();

    // Page margins are applied via the document's default section property
    // docx-rs handles this through the document's built-in section

    // Set up numbering definitions for lists
    let bullet_abstract = AbstractNumbering::new(1)
        .add_level(
            Level::new(
                0,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("•"),
                LevelJc::new("left"),
            )
            .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None)
        )
        .add_level(
            Level::new(
                1,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("◦"),
                LevelJc::new("left"),
            )
            .indent(Some(1440), Some(SpecialIndentType::Hanging(360)), None, None)
        )
        .add_level(
            Level::new(
                2,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("▪"),
                LevelJc::new("left"),
            )
            .indent(Some(2160), Some(SpecialIndentType::Hanging(360)), None, None)
        );

    let ordered_abstract = AbstractNumbering::new(2)
        .add_level(
            Level::new(
                0,
                Start::new(1),
                NumberFormat::new("decimal"),
                LevelText::new("%1."),
                LevelJc::new("left"),
            )
            .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None)
        )
        .add_level(
            Level::new(
                1,
                Start::new(1),
                NumberFormat::new("lowerLetter"),
                LevelText::new("%2."),
                LevelJc::new("left"),
            )
            .indent(Some(1440), Some(SpecialIndentType::Hanging(360)), None, None)
        )
        .add_level(
            Level::new(
                2,
                Start::new(1),
                NumberFormat::new("lowerRoman"),
                LevelText::new("%3."),
                LevelJc::new("left"),
            )
            .indent(Some(2160), Some(SpecialIndentType::Hanging(360)), None, None)
        );

    doc = doc
        .add_abstract_numbering(bullet_abstract)
        .add_abstract_numbering(ordered_abstract)
        .add_numbering(Numbering::new(1, 1))
        .add_numbering(Numbering::new(2, 2));

    // Walk the AST and build the document
    convert_children(root, &mut doc, 0, style, &alignments);

    let mut buf = Vec::new();
    doc.build()
        .pack(&mut std::io::Cursor::new(&mut buf))
        .map_err(|e| ConversionError::DocxError(e.to_string()))?;

    Ok(buf)
}

/// Recursively convert AST children to DOCX elements.
fn convert_children<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    doc: &mut Docx,
    list_depth: usize,
    style: &DocxStyle,
    alignments: &std::collections::HashMap<String, AlignmentType>,
) {
    for child in node.children() {
        let ast = child.data.borrow();
        match &ast.value {
            NodeValue::Heading(heading) => {
                let level = heading.level;
                let mut para = Paragraph::new();
                let (heading_style, size) = match level {
                    1 => ("Heading1", style.heading1_half_pts()),
                    2 => ("Heading2", style.heading2_half_pts()),
                    3 => ("Heading3", style.heading3_half_pts()),
                    _ => ("Heading4", style.heading4_half_pts()),
                };
                para = para.style(heading_style);

                drop(ast);
                let runs = collect_inline_runs(child, style);
                for mut run in runs {
                    run = run.size(size).fonts(font(&style.body_font));
                    if level <= 2 {
                        run = run.bold();
                    }
                    para = para.add_run(run);
                }

                // Check alignment
                let first_text = get_first_text(child);
                if let Some(align) = alignments.get(first_text.trim()) {
                    para = para.align(*align);
                }

                para = para.line_spacing(
                    LineSpacing::new()
                        .before(240)
                        .after(120)
                );

                *doc = std::mem::take(doc).add_paragraph(para);
            }

            NodeValue::Paragraph => {
                drop(ast);
                let runs = collect_inline_runs(child, style);
                let mut para = Paragraph::new();

                // Apply body font and spacing
                para = para.line_spacing(
                    LineSpacing::new()
                        .line(style.line_spacing_twips())
                        .line_rule(LineSpacingType::Auto)
                        .after(160)
                );

                // Check if parent is a list item
                if let Some(parent) = child.parent() {
                    let parent_ast = parent.data.borrow();
                    if let NodeValue::Item(_) = &parent_ast.value {
                        // Check grandparent for list type
                        drop(parent_ast);
                        if let Some(grandparent) = parent.parent() {
                            let gp_ast = grandparent.data.borrow();
                            if let NodeValue::List(list) = &gp_ast.value {
                                let (num_id, _indent_level) = match list.list_type {
                                    ListType::Bullet => (1, list_depth),
                                    ListType::Ordered => (2, list_depth),
                                };
                                let depth = list_depth.min(2);
                                para = para.numbering(
                                    NumberingId::new(num_id),
                                    IndentLevel::new(depth),
                                );
                            }
                        }
                    }
                }

                // Check first text node for alignment match
                let first_text = get_first_text(child);
                if let Some(align) = alignments.get(first_text.trim()) {
                    para = para.align(*align);
                }

                for mut run in runs {
                    run = run.size(style.body_size_half_pts()).fonts(font(&style.body_font));
                    para = para.add_run(run);
                }

                *doc = std::mem::take(doc).add_paragraph(para);
            }

            NodeValue::BlockQuote => {
                drop(ast);
                // Collect paragraphs inside blockquote with indentation and left border
                for bq_child in child.children() {
                    let bq_ast = bq_child.data.borrow();
                    if let NodeValue::Paragraph = &bq_ast.value {
                        drop(bq_ast);
                        let runs = collect_inline_runs(bq_child, style);
                        let mut para = Paragraph::new()
                            .indent(Some(720), None, None, None)
                            .line_spacing(
                                LineSpacing::new()
                                    .line(style.line_spacing_twips())
                                    .line_rule(LineSpacingType::Auto)
                                    .after(160)
                            );

                        for mut run in runs {
                            run = run
                                .size(style.body_size_half_pts())
                                .fonts(font(&style.body_font))
                                .italic()
                                .color("555555");
                            para = para.add_run(run);
                        }

                        *doc = std::mem::take(doc).add_paragraph(para);
                    } else {
                        drop(bq_ast);
                        convert_children(bq_child, doc, list_depth, style, alignments);
                    }
                }
            }

            NodeValue::CodeBlock(code_block) => {
                let code_text = code_block.literal.clone();
                drop(ast);

                // Each line of code gets its own paragraph with monospace font and gray background
                for line in code_text.lines() {
                    let para = Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(line)
                                .size(style.code_size_half_pts())
                                .fonts(font(&style.code_font))
                                .color("333333")
                        )
                        .indent(Some(360), None, Some(360), None)
                        .line_spacing(
                            LineSpacing::new()
                                .line(240)
                                .line_rule(LineSpacingType::Auto)
                                .before(0)
                                .after(0)
                        );

                    *doc = std::mem::take(doc).add_paragraph(para);
                }

                // Add spacing paragraph after code block
                let spacer = Paragraph::new()
                    .line_spacing(LineSpacing::new().after(160));
                *doc = std::mem::take(doc).add_paragraph(spacer);
            }

            NodeValue::List(list) => {
                drop(ast);
                let new_depth = list_depth + if list_depth > 0 { 1 } else { 0 };
                for item_child in child.children() {
                    convert_children(item_child, doc, new_depth, style, alignments);
                }
            }

            NodeValue::Item(_) => {
                drop(ast);
                convert_children(child, doc, list_depth, style, alignments);
            }

            NodeValue::Table(_alignments) => {
                drop(ast);
                let mut rows: Vec<TableRow> = Vec::new();
                let mut is_header = true;

                for row_node in child.children() {
                    let row_ast = row_node.data.borrow();
                    if let NodeValue::TableRow(_is_header_row) = &row_ast.value {
                        drop(row_ast);
                        let mut cells: Vec<TableCell> = Vec::new();

                        for cell_node in row_node.children() {
                            let cell_ast = cell_node.data.borrow();
                            if let NodeValue::TableCell = &cell_ast.value {
                                drop(cell_ast);
                                let runs = collect_inline_runs(cell_node, style);
                                let mut para = Paragraph::new()
                                    .line_spacing(
                                        LineSpacing::new()
                                            .line(240)
                                            .line_rule(LineSpacingType::Auto)
                                            .before(40)
                                            .after(40)
                                    );

                                for mut run in runs {
                                    run = run.size(style.body_size_half_pts()).fonts(font(&style.body_font));
                                    if is_header {
                                        run = run.bold();
                                    }
                                    para = para.add_run(run);
                                }

                                let mut cell = TableCell::new()
                                    .add_paragraph(para)
                                    .vertical_align(VAlignType::Center);

                                if is_header {
                                    cell = cell.shading(Shading::new().fill("E8E8E8"));
                                }

                                cells.push(cell);
                            } else {
                                drop(cell_ast);
                            }
                        }

                        rows.push(TableRow::new(cells));
                        is_header = false;
                    } else {
                        drop(row_ast);
                    }
                }

                if !rows.is_empty() {
                    let table = Table::new(rows)
                        .set_borders(
                            TableBorders::new()
                                .set(TableBorder::new(TableBorderPosition::Top).border_type(BorderType::Single).size(4).color("999999"))
                                .set(TableBorder::new(TableBorderPosition::Bottom).border_type(BorderType::Single).size(4).color("999999"))
                                .set(TableBorder::new(TableBorderPosition::Left).border_type(BorderType::Single).size(4).color("999999"))
                                .set(TableBorder::new(TableBorderPosition::Right).border_type(BorderType::Single).size(4).color("999999"))
                                .set(TableBorder::new(TableBorderPosition::InsideH).border_type(BorderType::Single).size(4).color("CCCCCC"))
                                .set(TableBorder::new(TableBorderPosition::InsideV).border_type(BorderType::Single).size(4).color("CCCCCC"))
                        )
                        .width(9360, WidthType::Dxa)
                        .layout(TableLayoutType::Fixed);

                    *doc = std::mem::take(doc).add_table(table);

                    // Spacing after table
                    let spacer = Paragraph::new()
                        .line_spacing(LineSpacing::new().after(160));
                    *doc = std::mem::take(doc).add_paragraph(spacer);
                }
            }

            NodeValue::ThematicBreak => {
                drop(ast);
                // Render as a centered line of underscores to simulate HR
                let para = Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text("_______________________________________")
                            .color("CCCCCC")
                            .size(style.body_size_half_pts())
                    )
                    .align(AlignmentType::Center)
                    .line_spacing(
                        LineSpacing::new()
                            .before(240)
                            .after(240)
                    );
                *doc = std::mem::take(doc).add_paragraph(para);
            }

            NodeValue::FrontMatter(_) | NodeValue::Document => {
                drop(ast);
                convert_children(child, doc, list_depth, style, alignments);
            }

            NodeValue::HtmlBlock(html) => {
                // Render raw HTML blocks as plain text in a paragraph
                let text = strip_html_tags(&html.literal);
                if !text.trim().is_empty() {
                    drop(ast);
                    let para = Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(text.trim())
                                .size(style.body_size_half_pts())
                                .fonts(font(&style.body_font))
                        )
                        .line_spacing(
                            LineSpacing::new()
                                .line(style.line_spacing_twips())
                                .line_rule(LineSpacingType::Auto)
                                .after(160)
                        );
                    *doc = std::mem::take(doc).add_paragraph(para);
                }
            }

            NodeValue::FootnoteDefinition(_) => {
                drop(ast);
                let runs = collect_all_text(child);
                let text = runs.join("");
                if !text.trim().is_empty() {
                    let para = Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(format!("[footnote]: {}", text.trim()))
                                .size(style.body_size_half_pts() - 4)
                                .fonts(font(&style.body_font))
                                .color("666666")
                        )
                        .indent(Some(360), None, None, None)
                        .line_spacing(
                            LineSpacing::new()
                                .line(style.line_spacing_twips())
                                .line_rule(LineSpacingType::Auto)
                                .after(80)
                        );
                    *doc = std::mem::take(doc).add_paragraph(para);
                }
            }

            _ => {
                drop(ast);
            }
        }
    }
}

/// Collect inline runs from a node's children, handling nested formatting.
fn collect_inline_runs<'a>(node: &'a comrak::nodes::AstNode<'a>, style: &DocxStyle) -> Vec<Run> {
    let mut runs = Vec::new();
    let state = InlineState::default();
    walk_inline_nodes(node, &state, &mut runs, style);
    runs
}

fn walk_inline_nodes<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    state: &InlineState,
    runs: &mut Vec<Run>,
    style: &DocxStyle,
) {
    for child in node.children() {
        let ast = child.data.borrow();
        match &ast.value {
            NodeValue::Text(text) => {
                let mut run = Run::new().add_text(text.as_str());
                run = apply_inline_state(run, state);
                runs.push(run);
            }

            NodeValue::SoftBreak => {
                let mut run = Run::new().add_text(" ");
                run = apply_inline_state(run, state);
                runs.push(run);
            }

            NodeValue::LineBreak => {
                runs.push(Run::new().add_break(BreakType::TextWrapping));
            }

            NodeValue::Code(code) => {
                let mut run = Run::new()
                    .add_text(code.literal.as_str())
                    .fonts(font(&style.code_font))
                    .size(style.code_size_half_pts())
                    .color("C7254E")
                    .highlight("lightGray");
                if state.bold {
                    run = run.bold();
                }
                runs.push(run);
            }

            NodeValue::Strong => {
                let mut new_state = state.clone();
                new_state.bold = true;
                drop(ast);
                walk_inline_nodes(child, &new_state, runs, style);
                continue;
            }

            NodeValue::Emph => {
                let mut new_state = state.clone();
                new_state.italic = true;
                drop(ast);
                walk_inline_nodes(child, &new_state, runs, style);
                continue;
            }

            NodeValue::Strikethrough => {
                let mut new_state = state.clone();
                new_state.strikethrough = true;
                drop(ast);
                walk_inline_nodes(child, &new_state, runs, style);
                continue;
            }

            NodeValue::Link(link) => {
                let mut new_state = state.clone();
                new_state.link_url = Some(link.url.clone());
                drop(ast);
                walk_inline_nodes(child, &new_state, runs, style);
                continue;
            }

            NodeValue::FootnoteReference(_) => {
                let mut run = Run::new()
                    .add_text("[*]")
                    .size(style.body_size_half_pts() - 4)
                    .color("0563C1");
                run = apply_inline_state(run, state);
                runs.push(run);
            }

            NodeValue::TaskItem(checked) => {
                let checkbox = if checked.is_some() { "☑ " } else { "☐ " };
                let run = Run::new()
                    .add_text(checkbox)
                    .size(style.body_size_half_pts());
                runs.push(run);
            }

            _ => {
                drop(ast);
                walk_inline_nodes(child, state, runs, style);
                continue;
            }
        }
        drop(ast);
    }
}

fn apply_inline_state(mut run: Run, state: &InlineState) -> Run {
    if state.bold {
        run = run.bold();
    }
    if state.italic {
        run = run.italic();
    }
    if state.strikethrough {
        run = run.strike();
    }
    if state.link_url.is_some() {
        run = run.color("0563C1").underline("single");
    }
    run
}

/// Get the first text content from a node's children (for alignment lookup).
fn get_first_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    for child in node.children() {
        let ast = child.data.borrow();
        if let NodeValue::Text(text) = &ast.value {
            return text.clone();
        }
        drop(ast);
        let result = get_first_text(child);
        if !result.is_empty() {
            return result;
        }
    }
    String::new()
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn collect_all_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Vec<String> {
    let mut texts = Vec::new();
    for child in node.children() {
        let ast = child.data.borrow();
        if let NodeValue::Paragraph = &ast.value {
            drop(ast);
            for inline in child.children() {
                let inline_ast = inline.data.borrow();
                if let NodeValue::Text(t) = &inline_ast.value {
                    texts.push(t.clone());
                }
            }
        } else {
            drop(ast);
            texts.extend(collect_all_text(child));
        }
    }
    texts
}

/// Extract plain text from DOCX bytes (basic extraction).
pub fn docx_to_markdown(bytes: &[u8]) -> Result<String, ConversionError> {
    let doc = read_docx(bytes)
        .map_err(|e| ConversionError::DocxError(format!("{e:?}")))?;

    let mut lines = Vec::new();

    for child in doc.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
                let mut line = String::new();
                let mut is_heading = false;
                let mut heading_level = 0u8;

                if let Some(ref style) = para.property.style {
                    let style_id = &style.val;
                    if style_id.contains("Heading1") || style_id == "1" {
                        is_heading = true;
                        heading_level = 1;
                    } else if style_id.contains("Heading2") || style_id == "2" {
                        is_heading = true;
                        heading_level = 2;
                    } else if style_id.contains("Heading3") || style_id == "3" {
                        is_heading = true;
                        heading_level = 3;
                    } else if style_id.contains("Heading4") || style_id == "4" {
                        is_heading = true;
                        heading_level = 4;
                    }
                }

                let has_numbering = para.property.numbering_property.is_some();

                for child in &para.children {
                    if let ParagraphChild::Run(run) = child {
                        let is_bold = run.run_property.bold.is_some();
                        let is_italic = run.run_property.italic.is_some();
                        let is_strike = run.run_property.strike.is_some();

                        for run_child in &run.children {
                            if let RunChild::Text(text) = run_child {
                                let t = &text.text;
                                let mut formatted = t.clone();
                                if is_bold && !is_heading {
                                    formatted = format!("**{formatted}**");
                                }
                                if is_italic {
                                    formatted = format!("*{formatted}*");
                                }
                                if is_strike {
                                    formatted = format!("~~{formatted}~~");
                                }
                                line.push_str(&formatted);
                            }
                        }
                    }
                }

                if is_heading {
                    let prefix = "#".repeat(heading_level as usize);
                    lines.push(format!("{prefix} {line}"));
                } else if has_numbering {
                    lines.push(format!("- {line}"));
                } else {
                    lines.push(line);
                }
            }
            DocumentChild::Table(table) => {
                let mut table_rows: Vec<Vec<String>> = Vec::new();
                for row in &table.rows {
                    if let TableChild::TableRow(row) = row {
                        let mut cells: Vec<String> = Vec::new();
                        for cell in &row.cells {
                            if let TableRowChild::TableCell(cell) = cell {
                                let mut cell_text = String::new();
                                for child in &cell.children {
                                    if let TableCellContent::Paragraph(para) = child {
                                        for pc in &para.children {
                                            if let ParagraphChild::Run(run) = pc {
                                                for rc in &run.children {
                                                    if let RunChild::Text(text) = rc {
                                                        cell_text.push_str(&text.text);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                cells.push(cell_text);
                            }
                        }
                        table_rows.push(cells);
                    }
                }

                if !table_rows.is_empty() {
                    let col_count = table_rows.iter().map(|r| r.len()).max().unwrap_or(0);
                    for (i, row) in table_rows.iter().enumerate() {
                        let padded: Vec<String> = (0..col_count)
                            .map(|j| row.get(j).cloned().unwrap_or_default())
                            .collect();
                        lines.push(format!("| {} |", padded.join(" | ")));
                        if i == 0 {
                            let sep: Vec<&str> = (0..col_count).map(|_| "---").collect();
                            lines.push(format!("| {} |", sep.join(" | ")));
                        }
                    }
                    lines.push(String::new());
                }
            }
            _ => {}
        }
    }

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_valid_zip() {
        let bytes = markdown_to_docx("# Hello\n\nA paragraph.").unwrap();
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn roundtrip_heading() {
        let md = "# Test Heading";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("# Test Heading"), "got: {back}");
    }

    #[test]
    fn roundtrip_paragraph() {
        let md = "Just a paragraph.";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("Just a paragraph."), "got: {back}");
    }

    #[test]
    fn roundtrip_bold() {
        let md = "This has **bold** words.";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("**bold**"), "got: {back}");
    }

    #[test]
    fn roundtrip_italic() {
        let md = "This has *italic* words.";
        let docx_bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&docx_bytes).unwrap();
        assert!(back.contains("*italic*"), "got: {back}");
    }

    #[test]
    fn handles_empty_input() {
        let bytes = markdown_to_docx("").unwrap();
        assert_eq!(&bytes[..2], b"PK");
    }

    #[test]
    fn handles_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let bytes = markdown_to_docx(md).unwrap();
        assert!(bytes.len() > 100);
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("fn main()"), "got: {back}");
    }

    #[test]
    fn handles_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let bytes = markdown_to_docx(md).unwrap();
        assert!(bytes.len() > 100);
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("A"), "got: {back}");
        assert!(back.contains("1"), "got: {back}");
    }

    #[test]
    fn handles_blockquote() {
        let md = "> This is a quote";
        let bytes = markdown_to_docx(md).unwrap();
        assert!(bytes.len() > 100);
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("This is a quote"), "got: {back}");
    }

    #[test]
    fn handles_strikethrough() {
        let md = "This has ~~deleted~~ text.";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("~~deleted~~"), "got: {back}");
    }

    #[test]
    fn handles_nested_formatting() {
        let md = "This is ***bold and italic*** text.";
        let bytes = markdown_to_docx(md).unwrap();
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_bullet_list() {
        let md = "- Item 1\n- Item 2\n- Item 3";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("Item 1"), "got: {back}");
        assert!(back.contains("Item 2"), "got: {back}");
    }

    #[test]
    fn handles_ordered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("First"), "got: {back}");
    }

    #[test]
    fn handles_horizontal_rule() {
        let md = "Above\n\n---\n\nBelow";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("Above"), "got: {back}");
        assert!(back.contains("Below"), "got: {back}");
    }

    #[test]
    fn handles_inline_code() {
        let md = "Use the `println!` macro.";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("println!"), "got: {back}");
    }

    #[test]
    fn handles_links() {
        let md = "Visit [Rust](https://rust-lang.org) for more.";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("Rust"), "got: {back}");
    }

    #[test]
    fn handles_multiple_headings() {
        let md = "# H1\n\n## H2\n\n### H3\n\n#### H4";
        let bytes = markdown_to_docx(md).unwrap();
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("# H1"), "got: {back}");
        assert!(back.contains("## H2"), "got: {back}");
        assert!(back.contains("### H3"), "got: {back}");
    }

    #[test]
    fn college_paper_format() {
        let md = r#"# Introduction to Rust

## Overview

Rust is a systems programming language that focuses on **safety**, **concurrency**, and **performance**. It achieves memory safety without a garbage collector through its *ownership system*.

### Key Features

1. Zero-cost abstractions
2. Move semantics
3. Guaranteed memory safety

> "Rust is the most loved programming language" — Stack Overflow Developer Survey

### Code Example

```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

| Feature | Rust | C++ |
|---------|------|-----|
| Memory Safety | Yes | No |
| Garbage Collection | No | No |
| Concurrency | Safe | Manual |

---

## Conclusion

Rust provides a compelling alternative to C and C++ for systems programming with ~~no~~ minimal runtime overhead.
"#;
        let bytes = markdown_to_docx(md).unwrap();
        assert!(bytes.len() > 1000);
        let back = docx_to_markdown(&bytes).unwrap();
        assert!(back.contains("# Introduction to Rust"), "got: {back}");
        assert!(back.contains("## Overview"), "got: {back}");
        assert!(back.contains("**safety**"), "got: {back}");
        assert!(back.contains("*ownership system*"), "got: {back}");
    }
}
