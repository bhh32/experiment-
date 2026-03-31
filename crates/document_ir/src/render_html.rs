use crate::ir::*;
use shared::DocStyle;

/// Render a Document IR to HTML, respecting the given document style.
pub fn render_to_html(doc: &Document, style: &DocStyle) -> String {
    let mut result = String::new();

    // Header
    if let Some(ref hdr) = doc.header {
        let align = if hdr.alignment != Alignment::Left {
            format!(r#" style="text-align:{}""#, hdr.alignment.css_value())
        } else {
            String::new()
        };
        result.push_str(&format!(r#"<div class="doc-header"{align}>"#));
        render_runs(&hdr.runs, &mut result);
        result.push_str("</div>\n");
    }

    // Body — split at page breaks
    let mut pages: Vec<String> = Vec::new();
    let mut current_page = String::new();

    for block in &doc.children {
        if matches!(block, Block::PageBreak) {
            pages.push(current_page);
            current_page = String::new();
            continue;
        }
        render_block(block, &mut current_page, style);
    }
    pages.push(current_page);

    if pages.len() == 1 {
        result.push_str(&pages.into_iter().next().unwrap());
    } else {
        result.push_str(&pages.join(r#"<div class="page-break"></div>"#));
    }

    // Footer
    if let Some(ref ftr) = doc.footer {
        let align = if ftr.alignment != Alignment::Left {
            format!(r#" style="text-align:{}""#, ftr.alignment.css_value())
        } else {
            String::new()
        };
        result.push_str(&format!(r#"<div class="doc-footer"{align}>"#));
        render_runs(&ftr.runs, &mut result);
        result.push_str("</div>\n");
    }

    result
}

fn render_block(block: &Block, out: &mut String, style: &DocStyle) {
    match block {
        Block::Paragraph(p) => {
            let align = align_attr(&p.properties);
            out.push_str(&format!("<p{align}>"));
            render_runs(&p.runs, out);
            out.push_str("</p>\n");
        }
        Block::Heading(h) => {
            let level = h.level.min(6);
            let align = align_attr(&h.properties);
            out.push_str(&format!("<h{level}{align}>"));
            render_runs(&h.runs, out);
            out.push_str(&format!("</h{level}>\n"));
        }
        Block::List(list) => {
            let tag = if list.ordered { "ol" } else { "ul" };
            out.push_str(&format!("<{tag}>\n"));
            for item in &list.items {
                out.push_str("<li>");
                if let Some(checked) = item.checked {
                    let attr = if checked { " checked disabled" } else { " disabled" };
                    out.push_str(&format!(r#"<input type="checkbox"{attr}> "#));
                }
                render_runs(&item.runs, out);
                for child in &item.children {
                    render_block(child, out, style);
                }
                out.push_str("</li>\n");
            }
            out.push_str(&format!("</{tag}>\n"));
        }
        Block::Table(table) => {
            out.push_str("<table>\n<thead>\n<tr>\n");
            for cell in &table.header {
                let align = cell_align_attr(cell);
                let span = if cell.col_span > 1 { format!(r#" colspan="{}""#, cell.col_span) } else { String::new() };
                out.push_str(&format!("<th{align}{span}>"));
                render_runs(&cell.runs, out);
                out.push_str("</th>\n");
            }
            out.push_str("</tr>\n</thead>\n<tbody>\n");
            for row in &table.rows {
                out.push_str("<tr>\n");
                for cell in row {
                    let align = cell_align_attr(cell);
                    let span = if cell.col_span > 1 { format!(r#" colspan="{}""#, cell.col_span) } else { String::new() };
                    out.push_str(&format!("<td{align}{span}>"));
                    render_runs(&cell.runs, out);
                    out.push_str("</td>\n");
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody>\n</table>\n");
        }
        Block::CodeBlock(cb) => {
            let lang_class = if cb.language.is_empty() {
                String::new()
            } else {
                format!(r#" class="language-{}""#, html_escape(&cb.language))
            };
            out.push_str(&format!(
                "<pre><code{lang_class}>{}</code></pre>\n",
                html_escape(&cb.content)
            ));
        }
        Block::BlockQuote(inner) => {
            out.push_str("<blockquote>\n");
            for b in inner {
                render_block(b, out, style);
            }
            out.push_str("</blockquote>\n");
        }
        Block::Image(img) => {
            let alt = html_escape(&img.alt);
            let src = html_escape(&img.src);
            let title = if img.title.is_empty() {
                String::new()
            } else {
                format!(r#" title="{}""#, html_escape(&img.title))
            };
            let size = if img.width_px > 0 && img.height_px > 0 {
                format!(r#" width="{}" height="{}""#, img.width_px, img.height_px)
            } else if img.width_px > 0 {
                format!(r#" width="{}""#, img.width_px)
            } else {
                String::new()
            };
            out.push_str(&format!(r#"<p><img src="{src}" alt="{alt}"{title}{size}></p>{}"#, "\n"));
        }
        Block::ThematicBreak => {
            out.push_str("<hr>\n");
        }
        Block::PageBreak => {
            // Handled at the top level in render_to_html
        }
    }
}

fn render_runs(runs: &[Run], out: &mut String) {
    for run in runs {
        if run.text == "\n" {
            out.push_str("<br>");
            continue;
        }

        let mut open_tags = Vec::new();

        if let Some(ref url) = run.properties.link_url {
            out.push_str(&format!(r#"<a href="{}">"#, html_escape(url)));
            open_tags.push("a");
        }
        if run.properties.bold {
            out.push_str("<strong>");
            open_tags.push("strong");
        }
        if run.properties.italic {
            out.push_str("<em>");
            open_tags.push("em");
        }
        if run.properties.strikethrough {
            out.push_str("<del>");
            open_tags.push("del");
        }
        if run.properties.code {
            out.push_str("<code>");
            open_tags.push("code");
        }

        out.push_str(&html_escape(&run.text));

        for tag in open_tags.into_iter().rev() {
            out.push_str(&format!("</{tag}>"));
        }
    }
}

fn cell_align_attr(cell: &TableCell) -> String {
    if cell.alignment == Alignment::Left {
        String::new()
    } else {
        format!(r#" style="text-align:{}""#, cell.alignment.css_value())
    }
}

fn align_attr(props: &ParaProperties) -> String {
    if props.alignment == Alignment::Left {
        String::new()
    } else {
        format!(r#" style="text-align:{}""#, props.alignment.css_value())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_markdown;

    fn render(md: &str) -> String {
        let doc = parse_markdown(md);
        let style = DocStyle::default();
        render_to_html(&doc, &style)
    }

    #[test]
    fn heading_html() {
        let html = render("# Hello");
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn bold_html() {
        let html = render("**bold**");
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn centered_heading_html() {
        let html = render("{center}# Title");
        assert!(html.contains(r#"<h1 style="text-align:center">Title</h1>"#));
    }

    #[test]
    fn centered_after_blanks() {
        let html = render("\n\n\n\n{center}# Title\n{center}## Sub");
        assert!(html.contains(r#"style="text-align:center">Title"#));
        assert!(html.contains(r#"style="text-align:center">Sub"#));
    }

    #[test]
    fn page_break_splits() {
        let html = render("Before\n\n{pagebreak}\n\nAfter");
        assert!(html.contains(r#"<div class="page-break"></div>"#));
    }

    #[test]
    fn table_html() {
        let html = render("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn code_block_escapes() {
        let html = render("```\n<script>alert('xss')</script>\n```");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn list_html() {
        let html = render("- Item 1\n- Item 2");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>Item 1</li>"));
    }

    #[test]
    fn ordered_list_html() {
        let html = render("1. First\n2. Second");
        assert!(html.contains("<ol>"));
    }

    #[test]
    fn blockquote_html() {
        let html = render("> quoted text");
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn link_html() {
        let html = render("[click](https://example.com)");
        assert!(html.contains(r#"<a href="https://example.com">click</a>"#));
    }

    #[test]
    fn line_break_html() {
        let html = render("line one\nline two");
        assert!(html.contains("<br>"));
    }

    #[test]
    fn thematic_break() {
        let html = render("---");
        assert!(html.contains("<hr>"));
    }

    #[test]
    fn header_renders() {
        let html = render("{header:My Header}\n\n# Body");
        assert!(html.contains(r#"<div class="doc-header">"#));
        assert!(html.contains("My Header"));
    }

    #[test]
    fn centered_header_renders() {
        let html = render("{header:center:Centered}\n\nBody");
        assert!(html.contains(r#"style="text-align:center""#));
        assert!(html.contains("Centered"));
    }

    #[test]
    fn footer_renders() {
        let html = render("{footer:Page 1}\n\nBody");
        assert!(html.contains(r#"<div class="doc-footer">"#));
        assert!(html.contains("Page 1"));
    }

    #[test]
    fn image_renders() {
        let html = render("![alt](pic.png)");
        assert!(html.contains(r#"<img src="pic.png" alt="alt""#));
    }

    #[test]
    fn image_with_title() {
        let html = render("![photo](img.jpg \"My Photo\")");
        assert!(html.contains(r#"title="My Photo""#));
    }

    #[test]
    fn table_column_alignment_html() {
        let md = "| L | C | R |\n|:--|:-:|--:|\n| a | b | c |";
        let html = render(md);
        assert!(html.contains(r#"style="text-align:center""#));
        assert!(html.contains(r#"style="text-align:right""#));
    }
}
