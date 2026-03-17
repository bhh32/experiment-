//! HTML generator — produces clean, semantic HTML5 output.
//!
//! Converts a Document model to a valid HTML5 document with inline CSS styling.

use crate::HtmlError;
use rw_document::{
    Block, Document, Inline,
    block::TableBlock,
    inline::BreakType,
    properties::{Alignment, UnderlineStyle},
};
use std::path::Path;

/// Write a Document to an HTML file.
pub fn write_html(doc: &Document, path: &Path) -> Result<(), HtmlError> {
    let html = generate_html(doc);
    std::fs::write(path, html)?;
    Ok(())
}

/// Generate an HTML string from a Document.
pub fn generate_html(doc: &Document) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

    // Title
    if let Some(ref title) = doc.metadata.title {
        html.push_str(&format!("<title>{}</title>\n", html_escape(title)));
    } else {
        html.push_str("<title>Document</title>\n");
    }

    // Metadata
    if let Some(ref author) = doc.metadata.author {
        html.push_str(&format!(
            "<meta name=\"author\" content=\"{}\">\n",
            html_escape(author)
        ));
    }

    // Default stylesheet
    html.push_str("<style>\n");
    html.push_str(DEFAULT_CSS);
    html.push_str("</style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");

    // Body content
    for section in &doc.sections {
        for block in &section.content {
            write_block_html(block, &mut html);
        }
    }

    html.push_str("</body>\n");
    html.push_str("</html>\n");
    html
}

fn write_block_html(block: &Block, html: &mut String) {
    match block {
        Block::Paragraph(para) => {
            let style_name = para.properties.paragraph_style.as_deref().unwrap_or("Normal");
            let outline_level = para.properties.outline_level;

            // Determine HTML tag
            let tag = if let Some(level) = outline_level {
                if (1..=6).contains(&level) {
                    format!("h{}", level)
                } else {
                    "p".to_string()
                }
            } else if style_name.starts_with("Heading") || style_name.starts_with("heading") {
                // Try to extract heading level
                let level_char = style_name.chars().last().unwrap_or('1');
                if let Some(level) = level_char.to_digit(10) {
                    if (1..=6).contains(&level) {
                        format!("h{}", level)
                    } else {
                        "p".to_string()
                    }
                } else {
                    "p".to_string()
                }
            } else {
                "p".to_string()
            };

            // Build style attribute
            let mut styles: Vec<String> = Vec::new();
            if let Some(align) = para.properties.alignment {
                let align_str = match align {
                    Alignment::Left => "left",
                    Alignment::Center => "center",
                    Alignment::Right => "right",
                    Alignment::Justify => "justify",
                    Alignment::Distribute => "justify",
                };
                styles.push(format!("text-align: {}", align_str));
            }

            let style_attr = if styles.is_empty() {
                String::new()
            } else {
                format!(" style=\"{}\"", styles.join("; "))
            };

            html.push_str(&format!("<{}{}>\n", tag, style_attr));

            // Inline content
            for inline in &para.content {
                write_inline_html(inline, html);
            }

            html.push_str(&format!("</{}>\n", tag));
        }
        Block::Table(tbl) => {
            write_table_html(tbl, html);
        }
        Block::HorizontalRule(_) => {
            html.push_str("<hr/>\n");
        }
        Block::SubSection(sub) => {
            html.push_str("<div>\n");
            for block in &sub.content {
                write_block_html(block, html);
            }
            html.push_str("</div>\n");
        }
        _ => {}
    }
}

fn write_inline_html(inline: &Inline, html: &mut String) {
    match inline {
        Inline::Text(run) => {
            let props = &run.properties;

            // Build style and tag wrappers
            let mut open_tags = String::new();
            let mut close_tags = String::new();
            let mut styles: Vec<String> = Vec::new();

            if props.bold == Some(true) {
                open_tags.push_str("<b>");
                close_tags.insert_str(0, "</b>");
            }
            if props.italic == Some(true) {
                open_tags.push_str("<i>");
                close_tags.insert_str(0, "</i>");
            }
            if props.underline.is_some() {
                open_tags.push_str("<u>");
                close_tags.insert_str(0, "</u>");
            }
            if let Some(ref color) = props.color {
                styles.push(format!("color: #{:02X}{:02X}{:02X}", color.r, color.g, color.b));
            }
            if let Some(size) = props.font_size {
                // Convert half-points to pt
                let pt = size as f64 / 2.0;
                styles.push(format!("font-size: {}pt", pt));
            }
            if let Some(ref font) = props.font_family {
                styles.push(format!("font-family: '{}'", html_escape(font)));
            }

            if !styles.is_empty() {
                let style_str = styles.join("; ");
                open_tags = format!("<span style=\"{}\">{}", style_str, open_tags);
                close_tags.push_str("</span>");
            }

            html.push_str(&open_tags);
            html.push_str(&html_escape(&run.text));
            html.push_str(&close_tags);
        }
        Inline::Break(break_type) => match break_type {
            BreakType::Line => html.push_str("<br/>\n"),
            BreakType::Page => html.push_str("<hr class=\"page-break\"/>\n"),
            BreakType::Column => html.push_str("<br class=\"col-break\"/>\n"),
        },
        Inline::Tab => {
            html.push_str("&nbsp;&nbsp;&nbsp;&nbsp;");
        }
        Inline::NonBreakingSpace => {
            html.push_str("&nbsp;");
        }
        Inline::Hyperlink(link) => {
            let target = match &link.target {
                rw_document::inline::HyperlinkTarget::Url(url) => url.clone(),
                rw_document::inline::HyperlinkTarget::Bookmark(bm) => format!("#{}", bm),
                rw_document::inline::HyperlinkTarget::Email { address, subject } => {
                    if let Some(subj) = subject {
                        format!("mailto:{}?subject={}", address, subj)
                    } else {
                        format!("mailto:{}", address)
                    }
                }
            };
            html.push_str(&format!("<a href=\"{}\">", html_escape(&target)));
            for inner in &link.content {
                write_inline_html(inner, html);
            }
            html.push_str("</a>");
        }
        Inline::Image(img) => {
            let src = match &img.source {
                rw_document::inline::ImageSource::File(path) => html_escape(path),
                rw_document::inline::ImageSource::Url(url) => html_escape(url),
                rw_document::inline::ImageSource::Embedded { data_id, mime_type } => {
                    format!("data:{};base64,{}", mime_type, data_id)
                }
            };
            let alt = img.alt_text.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\" />",
                src,
                html_escape(alt)
            ));
        }
        _ => {}
    }
}

fn write_table_html(tbl: &TableBlock, html: &mut String) {
    html.push_str("<table>\n");
    let rows = tbl.rows as usize;
    let cols = tbl.cols as usize;
    if cols == 0 {
        html.push_str("</table>\n");
        return;
    }

    for row_idx in 0..rows {
        html.push_str("<tr>\n");
        for col_idx in 0..cols {
            let cell_idx = row_idx * cols + col_idx;
            html.push_str("<td>");
            if let Some(cell) = tbl.cells.get(cell_idx) {
                for block in &cell.content {
                    write_block_html(block, html);
                }
            }
            html.push_str("</td>\n");
        }
        html.push_str("</tr>\n");
    }

    html.push_str("</table>\n");
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const DEFAULT_CSS: &str = r#"
body {
    font-family: 'Liberation Serif', Georgia, serif;
    font-size: 12pt;
    line-height: 1.5;
    max-width: 800px;
    margin: 0 auto;
    padding: 1em;
    color: #000;
}
h1 { font-size: 2em; margin: 0.67em 0; }
h2 { font-size: 1.5em; margin: 0.75em 0; }
h3 { font-size: 1.17em; margin: 0.83em 0; }
h4 { font-size: 1em; margin: 1.12em 0; }
h5 { font-size: 0.83em; margin: 1.5em 0; }
h6 { font-size: 0.75em; margin: 1.67em 0; }
p { margin: 0.5em 0; }
table { border-collapse: collapse; width: 100%; }
td, th { border: 1px solid #999; padding: 4px 8px; }
hr.page-break { border: none; page-break-after: always; }
"#;
