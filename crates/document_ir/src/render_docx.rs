use docx_rs::*;
use shared::DocStyle;

use crate::ir;

fn font(name: &str) -> RunFonts {
    RunFonts::new().ascii(name).hi_ansi(name).cs(name).east_asia(name)
}

fn to_docx_align(align: ir::Alignment) -> AlignmentType {
    match align {
        ir::Alignment::Left => AlignmentType::Left,
        ir::Alignment::Center => AlignmentType::Center,
        ir::Alignment::Right => AlignmentType::Right,
        ir::Alignment::Justify => AlignmentType::Justified,
    }
}

/// Render a Document IR to DOCX bytes.
pub fn render_to_docx(doc: &ir::Document, style: &DocStyle) -> Result<Vec<u8>, String> {
    let mut docx = Docx::new();

    // Set up numbering for lists
    let bullet_abstract = AbstractNumbering::new(1)
        .add_level(
            Level::new(0, Start::new(1), NumberFormat::new("bullet"), LevelText::new("•"), LevelJc::new("left"))
                .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None),
        )
        .add_level(
            Level::new(1, Start::new(1), NumberFormat::new("bullet"), LevelText::new("◦"), LevelJc::new("left"))
                .indent(Some(1440), Some(SpecialIndentType::Hanging(360)), None, None),
        )
        .add_level(
            Level::new(2, Start::new(1), NumberFormat::new("bullet"), LevelText::new("▪"), LevelJc::new("left"))
                .indent(Some(2160), Some(SpecialIndentType::Hanging(360)), None, None),
        );

    let ordered_abstract = AbstractNumbering::new(2)
        .add_level(
            Level::new(0, Start::new(1), NumberFormat::new("decimal"), LevelText::new("%1."), LevelJc::new("left"))
                .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None),
        )
        .add_level(
            Level::new(1, Start::new(1), NumberFormat::new("lowerLetter"), LevelText::new("%2."), LevelJc::new("left"))
                .indent(Some(1440), Some(SpecialIndentType::Hanging(360)), None, None),
        )
        .add_level(
            Level::new(2, Start::new(1), NumberFormat::new("lowerRoman"), LevelText::new("%3."), LevelJc::new("left"))
                .indent(Some(2160), Some(SpecialIndentType::Hanging(360)), None, None),
        );

    docx = docx
        .add_abstract_numbering(bullet_abstract)
        .add_abstract_numbering(ordered_abstract)
        .add_numbering(Numbering::new(1, 1))
        .add_numbering(Numbering::new(2, 2));

    // Header
    if let Some(ref hdr) = doc.header {
        let mut hdr_para = Paragraph::new()
            .align(to_docx_align(hdr.alignment));
        for run in &hdr.runs {
            hdr_para = hdr_para.add_run(make_run(run, style, style.body_size_half_pts()));
        }
        let header = Header::new().add_paragraph(hdr_para);
        docx = docx.header(header);
    }

    // Footer
    if let Some(ref ftr) = doc.footer {
        let mut ftr_para = Paragraph::new()
            .align(to_docx_align(ftr.alignment));
        for run in &ftr.runs {
            ftr_para = ftr_para.add_run(make_run(run, style, style.body_size_half_pts()));
        }
        let footer = Footer::new().add_paragraph(ftr_para);
        docx = docx.footer(footer);
    }

    // Render blocks
    render_blocks(&doc.children, &mut docx, style, 0);

    let mut buf = Vec::new();
    docx.build()
        .pack(&mut std::io::Cursor::new(&mut buf))
        .map_err(|e| e.to_string())?;

    Ok(buf)
}

fn render_blocks(blocks: &[ir::Block], docx: &mut Docx, style: &DocStyle, list_depth: usize) {
    let mut page_break_next = false;

    for block in blocks {
        match block {
            ir::Block::PageBreak => {
                page_break_next = true;
            }
            ir::Block::Paragraph(p) => {
                let mut para = Paragraph::new();
                para = para.align(to_docx_align(p.properties.alignment));

                if page_break_next {
                    para = para.page_break_before(true);
                    page_break_next = false;
                }

                for run in &p.runs {
                    para = para.add_run(make_run(run, style, style.body_size_half_pts()));
                }

                *docx = std::mem::take(docx).add_paragraph(para);
            }
            ir::Block::Heading(h) => {
                let (style_name, half_pts) = match h.level {
                    1 => ("Heading1", style.heading1_half_pts()),
                    2 => ("Heading2", style.heading2_half_pts()),
                    3 => ("Heading3", style.heading3_half_pts()),
                    _ => ("Heading4", style.heading4_half_pts()),
                };

                let mut para = Paragraph::new()
                    .style(style_name)
                    .align(to_docx_align(h.properties.alignment));

                if page_break_next {
                    para = para.page_break_before(true);
                    page_break_next = false;
                }

                for run in &h.runs {
                    let mut r = make_run(run, style, half_pts);
                    if h.level <= 2 {
                        r = r.bold();
                    }
                    para = para.add_run(r);
                }

                *docx = std::mem::take(docx).add_paragraph(para);
            }
            ir::Block::List(list) => {
                let num_id = if list.ordered { 2 } else { 1 };
                render_list_items(&list.items, docx, style, num_id, list_depth);
            }
            ir::Block::Table(table) => {
                let mut rows = Vec::new();

                // Header row
                let mut header_cells = Vec::new();
                for cell in &table.header {
                    let mut para = Paragraph::new()
                        .align(to_docx_align(cell.alignment));
                    for run in &cell.runs {
                        para = para.add_run(make_run(run, style, style.body_size_half_pts()).bold());
                    }
                    let shading_color = cell.shading.as_deref().unwrap_or("e8e8e8");
                    let mut tc = TableCell::new()
                        .add_paragraph(para)
                        .shading(Shading::new().fill(shading_color));
                    if cell.col_span > 1 {
                        tc = tc.grid_span(cell.col_span as usize);
                    }
                    if cell.row_span > 1 {
                        tc = tc.vertical_merge(VMergeType::Restart);
                    }
                    header_cells.push(tc);
                }
                rows.push(TableRow::new(header_cells));

                // Data rows
                for row in &table.rows {
                    let mut cells = Vec::new();
                    for cell in row {
                        let mut para = Paragraph::new()
                            .align(to_docx_align(cell.alignment));
                        for run in &cell.runs {
                            para = para.add_run(make_run(run, style, style.body_size_half_pts()));
                        }
                        let mut tc = TableCell::new().add_paragraph(para);
                        if cell.col_span > 1 {
                            tc = tc.grid_span(cell.col_span as usize);
                        }
                        cells.push(tc);
                    }
                    rows.push(TableRow::new(cells));
                }

                let tbl = Table::new(rows)
                    .set_borders(
                        TableBorders::new()
                            .set(TableBorder::new(TableBorderPosition::Top).border_type(BorderType::Single).size(4).color("999999"))
                            .set(TableBorder::new(TableBorderPosition::Bottom).border_type(BorderType::Single).size(4).color("999999"))
                            .set(TableBorder::new(TableBorderPosition::Left).border_type(BorderType::Single).size(4).color("999999"))
                            .set(TableBorder::new(TableBorderPosition::Right).border_type(BorderType::Single).size(4).color("999999"))
                            .set(TableBorder::new(TableBorderPosition::InsideH).border_type(BorderType::Single).size(4).color("cccccc"))
                            .set(TableBorder::new(TableBorderPosition::InsideV).border_type(BorderType::Single).size(4).color("cccccc")),
                    );

                *docx = std::mem::take(docx).add_table(tbl);
            }
            ir::Block::CodeBlock(cb) => {
                let code_half_pts = (style.code_size_pt() * 2.0) as usize;
                for line in cb.content.lines() {
                    let run = Run::new()
                        .add_text(line)
                        .size(code_half_pts)
                        .fonts(font(&style.code_font));
                    *docx = std::mem::take(docx).add_paragraph(Paragraph::new().add_run(run));
                }
            }
            ir::Block::BlockQuote(inner) => {
                // Render blockquote as indented italic paragraphs
                for b in inner {
                    if let ir::Block::Paragraph(p) = b {
                        let mut para = Paragraph::new()
                            .indent(Some(720), None, None, None);
                        for run in &p.runs {
                            let r = make_run(run, style, style.body_size_half_pts()).italic();
                            para = para.add_run(r);
                        }
                        *docx = std::mem::take(docx).add_paragraph(para);
                    } else {
                        render_blocks(std::slice::from_ref(b), docx, style, list_depth);
                    }
                }
            }
            ir::Block::Image(img) => {
                // Images from URLs/paths — for DOCX we need the actual bytes.
                // If the src is a data URL or local file, we could embed it.
                // For now, add a placeholder paragraph with the alt text as a link.
                let alt = if img.alt.is_empty() { "[image]" } else { &img.alt };
                let mut run = Run::new()
                    .add_text(alt)
                    .size(style.body_size_half_pts())
                    .fonts(font(&style.body_font))
                    .color("0563C1")
                    .underline("single");
                let para = Paragraph::new().add_run(run);
                *docx = std::mem::take(docx).add_paragraph(para);
            }
            ir::Block::ThematicBreak => {
                *docx = std::mem::take(docx).add_paragraph(Paragraph::new());
            }
            ir::Block::SectionBreak => {
                // Section breaks in DOCX are handled via section properties
                *docx = std::mem::take(docx).add_paragraph(Paragraph::new());
            }
        }
    }
}

fn render_list_items(
    items: &[ir::ListItem],
    docx: &mut Docx,
    style: &DocStyle,
    num_id: usize,
    depth: usize,
) {
    for item in items {
        let mut para = Paragraph::new()
            .numbering(NumberingId::new(num_id), IndentLevel::new(depth));

        if let Some(checked) = item.checked {
            let prefix = if checked { "☑ " } else { "☐ " };
            para = para.add_run(
                Run::new()
                    .add_text(prefix)
                    .size(style.body_size_half_pts())
                    .fonts(font(&style.body_font)),
            );
        }

        for run in &item.runs {
            para = para.add_run(make_run(run, style, style.body_size_half_pts()));
        }

        *docx = std::mem::take(docx).add_paragraph(para);

        // Nested blocks (sub-lists)
        for child in &item.children {
            if let ir::Block::List(sub_list) = child {
                let sub_num_id = if sub_list.ordered { 2 } else { 1 };
                render_list_items(&sub_list.items, docx, style, sub_num_id, depth + 1);
            }
        }
    }
}

fn make_run(run: &ir::Run, style: &DocStyle, half_pts: usize) -> Run {
    if run.text == "\n" {
        return Run::new().add_break(BreakType::TextWrapping);
    }

    // Footnote reference: superscript number
    if let Some(id) = run.properties.footnote_ref {
        return Run::new()
            .add_text(format!("[{id}]"))
            .size(half_pts)
            .fonts(font(&style.body_font))
            .bold();
    }

    let run_font = run.properties.font.as_deref().unwrap_or(&style.body_font);
    let run_half_pts = run.properties.size_pt
        .map(|pt| (pt * 2.0) as usize)
        .unwrap_or(half_pts);

    let mut r = Run::new()
        .add_text(&run.text)
        .size(run_half_pts)
        .fonts(font(run_font));

    if run.properties.bold {
        r = r.bold();
    }
    if run.properties.italic {
        r = r.italic();
    }
    if run.properties.underline {
        r = r.underline("single");
    }
    if run.properties.strikethrough {
        r = r.strike();
    }
    // Note: docx-rs 0.4 doesn't expose vert_align on Run builder.
    // Superscript/subscript render correctly in HTML preview but
    // are omitted from DOCX export until docx-rs adds support.
    if run.properties.code {
        let code_half = (style.code_size_pt() * 2.0) as usize;
        r = r.size(code_half).fonts(font(&style.code_font));
    }
    if let Some(ref color) = run.properties.color {
        r = r.color(color);
    }
    if let Some(ref hl) = run.properties.highlight {
        r = r.highlight(hl);
    }
    if run.properties.link_url.is_some() {
        r = r.color("0563C1").underline("single");
    }

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_markdown;

    fn to_docx_bytes(md: &str) -> Vec<u8> {
        let doc = parse_markdown(md);
        let style = DocStyle::default();
        render_to_docx(&doc, &style).unwrap()
    }

    #[test]
    fn produces_valid_zip() {
        let bytes = to_docx_bytes("# Hello\n\nA paragraph.");
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn roundtrip_heading() {
        let bytes = to_docx_bytes("# Test Heading");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_bold() {
        let bytes = to_docx_bytes("**bold** text");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_table() {
        let bytes = to_docx_bytes("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_list() {
        let bytes = to_docx_bytes("- Item 1\n- Item 2");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_code_block() {
        let bytes = to_docx_bytes("```\ncode\n```");
        assert!(bytes.len() > 100);
    }

    #[test]
    fn handles_empty() {
        let bytes = to_docx_bytes("");
        assert_eq!(&bytes[..2], b"PK");
    }

    #[test]
    fn handles_page_break() {
        let bytes = to_docx_bytes("Before\n\n{pagebreak}\n\nAfter");
        assert!(bytes.len() > 100);
    }
}
