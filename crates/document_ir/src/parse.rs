use comrak::{Arena, Options, parse_document};
use comrak::nodes::{NodeValue, ListType};

use crate::ir::*;

fn gfm_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.render.hardbreaks = true;
    opts
}

/// Parse markdown (with our custom alignment/pagebreak/header/footer extensions) into a Document IR.
pub fn parse_markdown(markdown: &str) -> Document {
    let preprocessed = preprocess(markdown);

    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed.clean_md, &gfm_options());

    let mut blocks = Vec::new();
    convert_children(root, &mut blocks, &preprocessed);

    let mut doc = Document::new();
    doc.children = blocks;
    doc.header = preprocessed.header;
    doc.footer = preprocessed.footer;
    doc
}

struct Preprocessed {
    clean_md: String,
    line_alignments: Vec<(String, Alignment)>,
    page_break_before: Vec<String>,
    header: Option<HeaderFooter>,
    footer: Option<HeaderFooter>,
}

/// Parse `{header:text}` or `{header:center:text}` syntax.
fn parse_header_footer(line: &str) -> Option<(&str, HeaderFooter)> {
    let trimmed = line.trim();
    for kind in &["header", "footer"] {
        let prefix = format!("{{{kind}:");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            if let Some(content) = rest.strip_suffix('}') {
                // Check for alignment: {header:center:text}
                let (alignment, text) = if let Some(after_center) = content.strip_prefix("center:") {
                    (Alignment::Center, after_center)
                } else if let Some(after_right) = content.strip_prefix("right:") {
                    (Alignment::Right, after_right)
                } else if let Some(after_left) = content.strip_prefix("left:") {
                    (Alignment::Left, after_left)
                } else {
                    (Alignment::Left, content)
                };
                return Some((kind, HeaderFooter {
                    runs: vec![Run::text(text.to_string())],
                    alignment,
                }));
            }
        }
    }
    None
}

fn preprocess(markdown: &str) -> Preprocessed {
    let mut clean_lines = Vec::new();
    let mut line_alignments = Vec::new();
    let mut page_break_before = Vec::new();
    let mut header = None;
    let mut footer = None;
    let mut consecutive_blanks = 0;
    let mut next_has_page_break = false;

    for line in markdown.lines() {
        // Header/footer directives
        if let Some((kind, hf)) = parse_header_footer(line) {
            match kind {
                "header" => header = Some(hf),
                "footer" => footer = Some(hf),
                _ => {}
            }
            continue;
        }

        if line.trim() == "{pagebreak}" {
            next_has_page_break = true;
            clean_lines.push(String::new());
            consecutive_blanks = 0;
            continue;
        }

        let (align, text) = toolbar::formatting::parse_alignment(line);

        if text.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks > 1 {
                clean_lines.push("\u{00a0}".to_string());
            } else {
                clean_lines.push(String::new());
            }
            continue;
        }

        if align != "left" {
            let display = text.trim().trim_start_matches('#').trim().to_string();
            if !display.is_empty() {
                line_alignments.push((display, Alignment::from_str(align)));
            }
        }

        if next_has_page_break {
            let display = text.trim().trim_start_matches('#').trim().to_string();
            if !display.is_empty() {
                page_break_before.push(display);
            }
            next_has_page_break = false;
        }

        consecutive_blanks = 0;
        clean_lines.push(text.to_string());
    }

    Preprocessed {
        clean_md: clean_lines.join("\n"),
        line_alignments,
        page_break_before,
        header,
        footer,
    }
}

/// Look up alignment for a block by matching its text content.
fn lookup_alignment(text: &str, pre: &Preprocessed) -> Alignment {
    let trimmed = text.trim();
    for (content, align) in &pre.line_alignments {
        if trimmed.contains(content.as_str()) {
            return *align;
        }
    }
    Alignment::Left
}

/// Check if a block should have a page break before it.
fn has_page_break(text: &str, pre: &Preprocessed) -> bool {
    let trimmed = text.trim();
    pre.page_break_before.iter().any(|c| trimmed.contains(c.as_str()))
}

/// Extract plain text from an AST node recursively.
fn extract_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.children() {
        let ast = child.data.borrow();
        match &ast.value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(c) => text.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {
                drop(ast);
                text.push_str(&extract_text(child));
            }
        }
    }
    text
}

/// Check if a paragraph contains only a single Image child; if so, extract it.
fn extract_standalone_image<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Option<Image> {
    let children: Vec<_> = node.children().collect();
    if children.len() == 1 {
        let ast = children[0].data.borrow();
        if let NodeValue::Image(ref link) = ast.value {
            let src = link.url.clone();
            let title = link.title.clone();
            drop(ast);
            let alt = extract_text(children[0]);
            return Some(Image {
                src,
                alt,
                title,
                width_px: 0,
                height_px: 0,
            });
        }
    }
    None
}

/// Collect inline runs from an AST node.
fn collect_runs<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Vec<Run> {
    let mut runs = Vec::new();
    collect_runs_inner(node, &mut runs, &RunProperties::default());
    runs
}

fn collect_runs_inner<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    runs: &mut Vec<Run>,
    inherited: &RunProperties,
) {
    for child in node.children() {
        let ast = child.data.borrow();
        match &ast.value {
            NodeValue::Text(t) => {
                runs.push(Run {
                    text: t.clone(),
                    properties: inherited.clone(),
                });
            }
            NodeValue::Code(c) => {
                let mut props = inherited.clone();
                props.code = true;
                runs.push(Run {
                    text: c.literal.clone(),
                    properties: props,
                });
            }
            NodeValue::Strong => {
                let mut props = inherited.clone();
                props.bold = true;
                drop(ast);
                collect_runs_inner(child, runs, &props);
            }
            NodeValue::Emph => {
                let mut props = inherited.clone();
                props.italic = true;
                drop(ast);
                collect_runs_inner(child, runs, &props);
            }
            NodeValue::Strikethrough => {
                let mut props = inherited.clone();
                props.strikethrough = true;
                drop(ast);
                collect_runs_inner(child, runs, &props);
            }
            NodeValue::Link(link) => {
                let mut props = inherited.clone();
                props.link_url = Some(link.url.clone());
                drop(ast);
                collect_runs_inner(child, runs, &props);
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                // With hardbreaks behavior, all breaks become line breaks
                runs.push(Run::line_break());
            }
            _ => {
                drop(ast);
                collect_runs_inner(child, runs, inherited);
            }
        }
    }
}

/// Walk the comrak AST and build IR blocks.
fn convert_children<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    blocks: &mut Vec<Block>,
    pre: &Preprocessed,
) {
    for child in node.children() {
        let ast = child.data.borrow();
        match &ast.value {
            NodeValue::Paragraph => {
                drop(ast);

                // Check if this paragraph is a standalone image
                if let Some(image) = extract_standalone_image(child) {
                    blocks.push(Block::Image(image));
                    continue;
                }

                let text = extract_text(child);
                let alignment = lookup_alignment(&text, pre);

                if has_page_break(&text, pre) {
                    blocks.push(Block::PageBreak);
                }

                let runs = collect_runs(child);
                blocks.push(Block::Paragraph(Paragraph {
                    runs,
                    properties: ParaProperties {
                        alignment,
                        ..Default::default()
                    },
                }));
            }
            NodeValue::Heading(heading) => {
                let level = heading.level;
                drop(ast);
                let text = extract_text(child);
                let alignment = lookup_alignment(&text, pre);

                if has_page_break(&text, pre) {
                    blocks.push(Block::PageBreak);
                }

                let runs = collect_runs(child);
                blocks.push(Block::Heading(Heading {
                    level,
                    runs,
                    properties: ParaProperties {
                        alignment,
                        ..Default::default()
                    },
                }));
            }
            NodeValue::List(_list) => {
                drop(ast);
                // Determine if ordered by checking first child
                let ordered = child.children().next().map_or(false, |item| {
                    let item_ast = item.data.borrow();
                    matches!(&item_ast.value, NodeValue::Item(i) if matches!(i.list_type, ListType::Ordered))
                });

                let mut items = Vec::new();
                for item_node in child.children() {
                    let item_ast = item_node.data.borrow();

                    // TaskItem replaces Item in comrak's AST for task lists
                    let task_checked = if let NodeValue::TaskItem(c) = &item_ast.value {
                        Some(c.is_some())
                    } else {
                        None
                    };
                    drop(item_ast);

                    let mut item_runs = Vec::new();
                    let mut item_children = Vec::new();

                    for sub in item_node.children() {
                        let sub_ast = sub.data.borrow();
                        match &sub_ast.value {
                            NodeValue::Paragraph => {
                                drop(sub_ast);
                                item_runs.extend(collect_runs(sub));
                            }
                            NodeValue::List(_) => {
                                drop(sub_ast);
                                let mut nested = Vec::new();
                                convert_children(sub, &mut nested, pre);
                                item_children.extend(nested);
                            }
                            _ => {
                                drop(sub_ast);
                            }
                        }
                    }

                    items.push(ListItem {
                        runs: item_runs,
                        children: item_children,
                        checked: task_checked,
                    });
                }

                blocks.push(Block::List(List { ordered, items }));
            }
            NodeValue::CodeBlock(cb) => {
                blocks.push(Block::CodeBlock(CodeBlock {
                    language: cb.info.clone(),
                    content: cb.literal.clone(),
                }));
            }
            NodeValue::BlockQuote => {
                drop(ast);
                let mut inner = Vec::new();
                convert_children(child, &mut inner, pre);
                blocks.push(Block::BlockQuote(inner));
            }
            NodeValue::ThematicBreak => {
                blocks.push(Block::ThematicBreak);
            }
            NodeValue::Table(table_node) => {
                // Extract column alignments from GFM syntax
                use comrak::nodes::TableAlignment;
                let col_alignments: Vec<Alignment> = table_node.alignments.iter().map(|a| {
                    match a {
                        TableAlignment::Center => Alignment::Center,
                        TableAlignment::Left => Alignment::Left,
                        TableAlignment::Right => Alignment::Right,
                        _ => Alignment::Left,
                    }
                }).collect();

                drop(ast);
                let mut header = Vec::new();
                let mut rows = Vec::new();
                let mut is_first_row = true;

                for row_node in child.children() {
                    let row_ast = row_node.data.borrow();
                    let is_header_row = matches!(&row_ast.value, NodeValue::TableRow(true));
                    drop(row_ast);

                    let mut cells = Vec::new();
                    for (col_idx, cell_node) in row_node.children().enumerate() {
                        let runs = collect_runs(cell_node);
                        let col_align = col_alignments.get(col_idx).copied().unwrap_or(Alignment::Left);
                        let mut cell = TableCell::new(runs, is_header_row || is_first_row);
                        cell.alignment = col_align;
                        cells.push(cell);
                    }

                    if is_header_row || is_first_row {
                        header = cells;
                        is_first_row = false;
                    } else {
                        rows.push(cells);
                    }
                }

                blocks.push(Block::Table(Table {
                    header,
                    rows,
                    properties: TableProperties {
                        width_pct: 100,
                        column_alignments: col_alignments,
                    },
                }));
            }
            _ => {
                drop(ast);
                convert_children(child, blocks, pre);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading() {
        let doc = parse_markdown("# Hello World");
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Block::Heading(h) => {
                assert_eq!(h.level, 1);
                assert_eq!(h.runs[0].text, "Hello World");
            }
            _ => panic!("expected heading"),
        }
    }

    #[test]
    fn parses_bold_italic() {
        let doc = parse_markdown("**bold** and *italic*");
        match &doc.children[0] {
            Block::Paragraph(p) => {
                assert!(p.runs.iter().any(|r| r.text == "bold" && r.properties.bold));
                assert!(p.runs.iter().any(|r| r.text == "italic" && r.properties.italic));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_centered_heading() {
        let doc = parse_markdown("{center}# My Title");
        match &doc.children[0] {
            Block::Heading(h) => {
                assert_eq!(h.properties.alignment, Alignment::Center);
                assert_eq!(h.runs[0].text, "My Title");
            }
            _ => panic!("expected centered heading"),
        }
    }

    #[test]
    fn parses_page_break() {
        let doc = parse_markdown("Before\n\n{pagebreak}\n\nAfter");
        let mut found_break = false;
        for block in &doc.children {
            if matches!(block, Block::PageBreak) {
                found_break = true;
            }
        }
        assert!(found_break, "expected page break");
    }

    #[test]
    fn parses_centered_after_blank_lines() {
        let doc = parse_markdown("\n\n\n\n\n{center}# Title\n{center}## Subtitle");
        let headings: Vec<_> = doc.children.iter().filter_map(|b| {
            if let Block::Heading(h) = b { Some(h) } else { None }
        }).collect();
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].properties.alignment, Alignment::Center);
        assert_eq!(headings[1].properties.alignment, Alignment::Center);
    }

    #[test]
    fn parses_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let doc = parse_markdown(md);
        match &doc.children[0] {
            Block::Table(t) => {
                assert_eq!(t.header.len(), 2);
                assert_eq!(t.rows.len(), 1);
            }
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn parses_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let doc = parse_markdown(md);
        match &doc.children[0] {
            Block::CodeBlock(cb) => {
                assert_eq!(cb.language, "rust");
                assert!(cb.content.contains("fn main()"));
            }
            _ => panic!("expected code block"),
        }
    }

    #[test]
    fn parses_list() {
        let md = "- Item 1\n- Item 2";
        let doc = parse_markdown(md);
        match &doc.children[0] {
            Block::List(l) => {
                assert!(!l.ordered);
                assert_eq!(l.items.len(), 2);
            }
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn blank_lines_create_spacing() {
        let doc = parse_markdown("\n\n\n\nContent");
        // Should have at least one paragraph containing "Content"
        let has_content = doc.children.iter().any(|b| {
            if let Block::Paragraph(p) = b {
                p.runs.iter().any(|r| r.text.contains("Content"))
            } else {
                false
            }
        });
        assert!(has_content, "expected content paragraph");
    }

    #[test]
    fn parses_task_list() {
        let doc = parse_markdown("- [x] Done\n- [ ] Todo");
        match &doc.children[0] {
            Block::List(l) => {
                assert_eq!(l.items.len(), 2);
                assert_eq!(l.items[0].checked, Some(true), "first item should be checked");
                assert_eq!(l.items[1].checked, Some(false), "second item should be unchecked");
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn right_alignment() {
        let doc = parse_markdown("{right}Right-aligned text");
        match &doc.children[0] {
            Block::Paragraph(p) => assert_eq!(p.properties.alignment, Alignment::Right),
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_header() {
        let doc = parse_markdown("{header:My Document Title}\n\n# Body");
        assert!(doc.header.is_some());
        assert_eq!(doc.header.as_ref().unwrap().runs[0].text, "My Document Title");
    }

    #[test]
    fn parses_centered_header() {
        let doc = parse_markdown("{header:center:Centered Header}\n\nBody text");
        let hdr = doc.header.as_ref().unwrap();
        assert_eq!(hdr.alignment, Alignment::Center);
        assert_eq!(hdr.runs[0].text, "Centered Header");
    }

    #[test]
    fn parses_footer() {
        let doc = parse_markdown("{footer:Page Footer}\n\nBody text");
        assert!(doc.footer.is_some());
        assert_eq!(doc.footer.as_ref().unwrap().runs[0].text, "Page Footer");
    }

    #[test]
    fn parses_right_footer() {
        let doc = parse_markdown("{footer:right:Page 1}\n\nBody");
        let ftr = doc.footer.as_ref().unwrap();
        assert_eq!(ftr.alignment, Alignment::Right);
    }

    #[test]
    fn parses_image() {
        let doc = parse_markdown("![alt text](image.png \"title\")");
        match &doc.children[0] {
            Block::Image(img) => {
                assert_eq!(img.src, "image.png");
                assert_eq!(img.alt, "alt text");
                assert_eq!(img.title, "title");
            }
            _ => panic!("expected image"),
        }
    }

    #[test]
    fn parses_table_column_alignments() {
        let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |";
        let doc = parse_markdown(md);
        match &doc.children[0] {
            Block::Table(t) => {
                assert_eq!(t.properties.column_alignments.len(), 3);
                assert_eq!(t.properties.column_alignments[0], Alignment::Left);
                assert_eq!(t.properties.column_alignments[1], Alignment::Center);
                assert_eq!(t.properties.column_alignments[2], Alignment::Right);
                // Cell alignment should match column alignment
                assert_eq!(t.header[1].alignment, Alignment::Center);
                assert_eq!(t.rows[0][2].alignment, Alignment::Right);
            }
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn header_and_footer_together() {
        let doc = parse_markdown("{header:center:Report}\n{footer:right:Confidential}\n\n# Body");
        assert!(doc.header.is_some());
        assert!(doc.footer.is_some());
        assert_eq!(doc.header.as_ref().unwrap().alignment, Alignment::Center);
        assert_eq!(doc.footer.as_ref().unwrap().alignment, Alignment::Right);
    }
}
