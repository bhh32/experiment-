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

/// Parse markdown (with our custom alignment/pagebreak extensions) into a Document IR.
pub fn parse_markdown(markdown: &str) -> Document {
    // First pass: strip our custom prefixes, track alignment per line
    let preprocessed = preprocess(markdown);

    // Second pass: parse clean markdown with comrak
    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed.clean_md, &gfm_options());

    // Third pass: walk AST and build IR
    let mut blocks = Vec::new();
    convert_children(root, &mut blocks, &preprocessed);

    Document { children: blocks }
}

struct Preprocessed {
    clean_md: String,
    /// (stripped_text_content, alignment) for lines with alignment prefixes
    line_alignments: Vec<(String, Alignment)>,
    /// Text content of paragraphs that should have a page break before them
    page_break_before: Vec<String>,
}

fn preprocess(markdown: &str) -> Preprocessed {
    let mut clean_lines = Vec::new();
    let mut line_alignments = Vec::new();
    let mut page_break_before = Vec::new();
    let mut consecutive_blanks = 0;
    let mut next_has_page_break = false;

    for line in markdown.lines() {
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
                // Extra blank lines become NBSP paragraphs for visible spacing
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
            NodeValue::Table(_) => {
                drop(ast);
                let mut header = Vec::new();
                let mut rows = Vec::new();
                let mut is_first_row = true;

                for row_node in child.children() {
                    let row_ast = row_node.data.borrow();
                    let is_header_row = matches!(&row_ast.value, NodeValue::TableRow(true));
                    drop(row_ast);

                    let mut cells = Vec::new();
                    for cell_node in row_node.children() {
                        let runs = collect_runs(cell_node);
                        cells.push(TableCell {
                            runs,
                            alignment: Alignment::Left,
                            is_header: is_header_row || is_first_row,
                        });
                    }

                    if is_header_row || is_first_row {
                        header = cells;
                        is_first_row = false;
                    } else {
                        rows.push(cells);
                    }
                }

                blocks.push(Block::Table(Table { header, rows }));
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
}
