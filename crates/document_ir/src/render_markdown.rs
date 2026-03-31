use crate::ir::*;

/// Render a Document IR back to markdown text.
/// This is the reverse of parse_markdown — used when importing .docx files
/// so the editor can display and edit the content.
pub fn render_to_markdown(doc: &Document) -> String {
    let mut out = String::new();

    // Header directive
    if let Some(ref hdr) = doc.header {
        let align = match hdr.alignment {
            Alignment::Center => "center:",
            Alignment::Right => "right:",
            _ => "",
        };
        let text = runs_to_text(&hdr.runs);
        out.push_str(&format!("{{header:{align}{text}}}\n"));
    }

    // Footer directive
    if let Some(ref ftr) = doc.footer {
        let align = match ftr.alignment {
            Alignment::Center => "center:",
            Alignment::Right => "right:",
            _ => "",
        };
        let text = runs_to_text(&ftr.runs);
        out.push_str(&format!("{{footer:{align}{text}}}\n"));
    }

    if doc.header.is_some() || doc.footer.is_some() {
        out.push('\n');
    }

    render_blocks(&doc.children, &mut out, 0);

    // Footnotes
    if !doc.footnotes.is_empty() {
        out.push('\n');
        for note in &doc.footnotes {
            let text = runs_to_text(&note.runs);
            out.push_str(&format!("[^{}]: {text}\n", note.id));
        }
    }

    out.trim_end().to_string()
}

fn render_blocks(blocks: &[Block], out: &mut String, depth: usize) {
    let mut prev_was_block = false;

    for block in blocks {
        match block {
            Block::PageBreak => {
                if prev_was_block {
                    out.push('\n');
                }
                out.push_str("{pagebreak}\n\n");
                prev_was_block = false;
                continue;
            }
            Block::SectionBreak => {
                out.push_str("\n---\n\n");
                prev_was_block = false;
                continue;
            }
            _ => {}
        }

        // Add blank line between blocks (paragraph separator)
        if prev_was_block {
            out.push('\n');
        }

        match block {
            Block::Heading(h) => {
                let prefix = align_prefix(h.properties.alignment);
                let hashes = "#".repeat(h.level as usize);
                let text = runs_to_markdown(&h.runs);
                out.push_str(&format!("{prefix}{hashes} {text}\n"));
            }
            Block::Paragraph(p) => {
                let prefix = align_prefix(p.properties.alignment);
                let text = runs_to_markdown(&p.runs);
                // Don't emit empty paragraphs as blank lines — they're already
                // handled by the inter-block spacing
                if !text.trim().is_empty() && text.trim() != "\u{00a0}" {
                    out.push_str(&format!("{prefix}{text}\n"));
                }
            }
            Block::List(list) => {
                render_list(list, out, depth);
            }
            Block::Table(table) => {
                render_table(table, out);
            }
            Block::CodeBlock(cb) => {
                let lang = &cb.language;
                out.push_str(&format!("```{lang}\n"));
                out.push_str(&cb.content);
                if !cb.content.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n");
            }
            Block::BlockQuote(inner) => {
                let mut inner_md = String::new();
                render_blocks(inner, &mut inner_md, depth);
                for line in inner_md.lines() {
                    out.push_str(&format!("> {line}\n"));
                }
            }
            Block::Image(img) => {
                let title = if img.title.is_empty() {
                    String::new()
                } else {
                    format!(" \"{}\"", img.title)
                };
                out.push_str(&format!("![{}]({}{})\n", img.alt, img.src, title));
            }
            Block::ThematicBreak => {
                out.push_str("---\n");
            }
            Block::PageBreak | Block::SectionBreak => {
                // Handled above
            }
        }
        prev_was_block = true;
    }
}

fn render_list(list: &List, out: &mut String, depth: usize) {
    let indent = "    ".repeat(depth);
    for (i, item) in list.items.iter().enumerate() {
        let marker = if list.ordered {
            format!("{}. ", i + 1)
        } else {
            "- ".to_string()
        };

        let checkbox = match item.checked {
            Some(true) => "[x] ",
            Some(false) => "[ ] ",
            None => "",
        };

        let text = runs_to_markdown(&item.runs);
        out.push_str(&format!("{indent}{marker}{checkbox}{text}\n"));

        // Nested blocks (sub-lists)
        for child in &item.children {
            if let Block::List(sub) = child {
                render_list(sub, out, depth + 1);
            }
        }
    }
}

fn render_table(table: &Table, out: &mut String) {
    if table.header.is_empty() {
        return;
    }

    // Header row
    out.push('|');
    for cell in &table.header {
        let text = runs_to_markdown(&cell.runs);
        out.push_str(&format!(" {text} |"));
    }
    out.push('\n');

    // Separator row with alignment
    out.push('|');
    for cell in &table.header {
        match cell.alignment {
            Alignment::Center => out.push_str(":---:|"),
            Alignment::Right => out.push_str("---:|"),
            _ => out.push_str("---|"),
        }
    }
    out.push('\n');

    // Data rows
    for row in &table.rows {
        out.push('|');
        for cell in row {
            let text = runs_to_markdown(&cell.runs);
            out.push_str(&format!(" {text} |"));
        }
        out.push('\n');
    }
}

/// Convert runs to plain text (for header/footer directives).
fn runs_to_text(runs: &[Run]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

/// Convert runs to markdown text with inline formatting.
fn runs_to_markdown(runs: &[Run]) -> String {
    let mut out = String::new();
    for run in runs {
        if run.text == "\n" {
            out.push('\n');
            continue;
        }

        let mut text = run.text.clone();

        if run.properties.bold {
            text = format!("**{text}**");
        }
        if run.properties.italic {
            text = format!("*{text}*");
        }
        if run.properties.strikethrough {
            text = format!("~~{text}~~");
        }
        if run.properties.code {
            text = format!("`{text}`");
        }
        if run.properties.underline {
            text = format!("__{text}__");
        }
        if let Some(ref url) = run.properties.link_url {
            text = format!("[{text}]({url})");
        }
        if let Some(ref color) = run.properties.color {
            text = format!("{{color:{color}}}{text}{{/color}}");
        }
        if let Some(ref highlight) = run.properties.highlight {
            text = format!("{{highlight:{highlight}}}{text}{{/highlight}}");
        }

        out.push_str(&text);
    }
    out
}

fn align_prefix(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Center => "{center}",
        Alignment::Right => "{right}",
        Alignment::Justify => "{justify}",
        Alignment::Left => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_markdown;

    fn roundtrip(md: &str) -> String {
        let doc = parse_markdown(md);
        render_to_markdown(&doc)
    }

    #[test]
    fn heading_roundtrip() {
        let result = roundtrip("# Hello World");
        assert!(result.contains("# Hello World"));
    }

    #[test]
    fn bold_roundtrip() {
        let result = roundtrip("**bold text**");
        assert!(result.contains("**bold text**"));
    }

    #[test]
    fn italic_roundtrip() {
        let result = roundtrip("*italic text*");
        assert!(result.contains("*italic text*"));
    }

    #[test]
    fn centered_heading_roundtrip() {
        let result = roundtrip("{center}# Title");
        assert!(result.contains("{center}# Title"));
    }

    #[test]
    fn page_break_roundtrip() {
        let result = roundtrip("Before\n\n{pagebreak}\n\nAfter");
        assert!(result.contains("{pagebreak}"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn list_roundtrip() {
        let result = roundtrip("- Item 1\n- Item 2");
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 2"));
    }

    #[test]
    fn ordered_list_roundtrip() {
        let result = roundtrip("1. First\n2. Second");
        assert!(result.contains("1. First"));
        assert!(result.contains("2. Second"));
    }

    #[test]
    fn code_block_roundtrip() {
        let result = roundtrip("```rust\nfn main() {}\n```");
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn table_roundtrip() {
        let result = roundtrip("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(result.contains("| A |"));
        assert!(result.contains("| 1 |"));
    }

    #[test]
    fn blockquote_roundtrip() {
        let result = roundtrip("> quoted text");
        assert!(result.contains("> "));
        assert!(result.contains("quoted text"));
    }

    #[test]
    fn image_roundtrip() {
        let result = roundtrip("![alt](pic.png)");
        assert!(result.contains("![alt](pic.png)"));
    }

    #[test]
    fn header_footer_roundtrip() {
        let result = roundtrip("{header:center:My Header}\n{footer:right:Page 1}\n\nBody text");
        assert!(result.contains("{header:center:My Header}"));
        assert!(result.contains("{footer:right:Page 1}"));
        assert!(result.contains("Body text"));
    }

    #[test]
    fn full_document_roundtrip() {
        let original = "{center}# Title\n\n{pagebreak}\n\n## Body\n\n**Bold** and *italic* text.\n\n- Item 1\n- Item 2";
        let result = roundtrip(original);
        assert!(result.contains("{center}# Title"));
        assert!(result.contains("{pagebreak}"));
        assert!(result.contains("## Body"));
        assert!(result.contains("**Bold**"));
        assert!(result.contains("*italic*"));
        assert!(result.contains("- Item 1"));
    }
}
