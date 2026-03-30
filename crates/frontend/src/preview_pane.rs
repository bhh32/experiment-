use dioxus::prelude::*;

#[component]
pub fn PreviewPane(
    content: Signal<String>,
    mode: Signal<String>,
    font_family: Signal<String>,
    line_height: Signal<f32>,
) -> Element {
    let html = use_memo(move || {
        let text = content.read().clone();
        let m = mode.read().clone();
        let ff = font_family.read().clone();
        let lh = *line_height.read();
        match m.as_str() {
            "docx" => render_styled_preview(&text, &ff, lh, "docx"),
            "odt" => render_styled_preview(&text, &ff, lh, "odt"),
            _ => render_styled_preview(&text, &ff, lh, "markdown"),
        }
    });

    rsx! {
        div {
            class: "preview-content",
            dangerous_inner_html: "{html}",
        }
    }
}

fn render_styled_preview(markdown: &str, font_family: &str, line_height: f32, mode: &str) -> String {
    // Pre-process alignment prefixes: strip them from markdown, track which
    // paragraphs need alignment, then post-process the HTML
    let (clean_md, alignments) = preprocess_alignments(markdown);
    let mut base_html = markdown_preview::render_preview(&clean_md);

    // Apply alignment to paragraphs in the HTML
    base_html = apply_alignments(&base_html, &alignments);

    let (body_font, heading_font, code_font, body_size, container_class) = match mode {
        "docx" => (
            font_family,
            font_family,
            "Consolas, monospace",
            "11pt",
            "docx-preview",
        ),
        "odt" => (
            if font_family == "Calibri" { "Liberation Serif" } else { font_family },
            if font_family == "Calibri" { "Liberation Sans" } else { font_family },
            "Liberation Mono, Courier New, monospace",
            "12pt",
            "odt-preview",
        ),
        _ => (
            font_family,
            font_family,
            "Consolas, monospace",
            "11pt",
            "md-preview",
        ),
    };

    format!(
        r#"<div class="{container_class}" style="font-family: '{body_font}', sans-serif; font-size: {body_size}; line-height: {line_height}; max-width: 6.5in; margin: 0 auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1);">
<style>
.{container_class} h1 {{ font-family: '{heading_font}', sans-serif; font-size: 2.36em; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; }}
.{container_class} h2 {{ font-family: '{heading_font}', sans-serif; font-size: 1.82em; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; }}
.{container_class} h3 {{ font-family: '{heading_font}', sans-serif; font-size: 1.27em; font-weight: normal; margin: 12pt 0 6pt 0; border: none; padding: 0; }}
.{container_class} h4 {{ font-family: '{heading_font}', sans-serif; font-size: 1em; font-weight: normal; margin: 10pt 0 5pt 0; border: none; padding: 0; }}
.{container_class} p {{ font-family: '{body_font}', sans-serif; font-size: {body_size}; line-height: {line_height}; margin: 0 0 8pt 0; }}
.{container_class} ul, .{container_class} ol {{ font-family: '{body_font}', sans-serif; font-size: {body_size}; line-height: {line_height}; margin: 0 0 8pt 0; padding-left: 0.5in; }}
.{container_class} li {{ margin: 0 0 2pt 0; }}
.{container_class} blockquote {{ margin: 0 0 8pt 0.5in; padding: 0; border: none; font-style: italic; color: #555; }}
.{container_class} blockquote p {{ font-style: italic; color: #555; }}
.{container_class} pre {{ font-family: {code_font}; font-size: 0.91em; line-height: 1.0; margin: 0 0 8pt 0; padding: 6pt 0.25in; background: none; border: none; border-left: 2pt solid #ccc; }}
.{container_class} pre code {{ font-family: {code_font}; font-size: 0.91em; color: #333; background: none; padding: 0; }}
.{container_class} code {{ font-family: {code_font}; font-size: 0.91em; color: #c7254e; background: #f0f0f0; padding: 1pt 3pt; border-radius: 2pt; }}
.{container_class} table {{ border-collapse: collapse; width: 100%; margin: 0 0 8pt 0; font-family: '{body_font}', sans-serif; font-size: {body_size}; }}
.{container_class} th {{ border: 1pt solid #999; padding: 2pt 6pt; background: #e8e8e8; font-weight: bold; text-align: left; }}
.{container_class} td {{ border: 1pt solid #ccc; padding: 2pt 6pt; text-align: left; }}
.{container_class} hr {{ border: none; border-bottom: 1pt solid #ccc; margin: 12pt 0; }}
.{container_class} a {{ color: #0563c1; text-decoration: underline; }}
.{container_class} strong {{ font-weight: bold; }}
.{container_class} em {{ font-style: italic; }}
.{container_class} del {{ text-decoration: line-through; }}
</style>
{base_html}</div>"#
    )
}

/// Strip alignment prefixes from markdown lines and collect which
/// paragraph indices need alignment. Returns (cleaned markdown, Vec<alignment>)
/// where alignments[i] is the alignment for the i-th paragraph.
fn preprocess_alignments(markdown: &str) -> (String, Vec<String>) {
    let mut clean_lines = Vec::new();
    let mut alignments = Vec::new();
    let mut current_align = "left".to_string();

    for line in markdown.lines() {
        let (align, text) = toolbar::formatting::parse_alignment(line);
        if align != "left" {
            current_align = align.to_string();
        }
        // Track alignment for each non-empty line that starts a paragraph
        if !text.trim().is_empty() && (clean_lines.is_empty() || clean_lines.last() == Some(&String::new())) {
            alignments.push(current_align.clone());
            if align == "left" {
                // Don't reset; keep default
            }
        }
        if text.trim().is_empty() {
            current_align = "left".to_string();
        }
        clean_lines.push(text.to_string());
    }

    (clean_lines.join("\n"), alignments)
}

/// Post-process HTML to wrap paragraphs/headings with alignment styles.
fn apply_alignments(html: &str, alignments: &[String]) -> String {
    let block_tags = ["<p>", "<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
    let mut result = String::with_capacity(html.len() + alignments.len() * 30);
    let mut para_idx = 0;

    for line in html.lines() {
        let trimmed = line.trim();
        let is_block = block_tags.iter().any(|tag| trimmed.starts_with(tag));

        if is_block {
            if let Some(align) = alignments.get(para_idx) {
                if align != "left" {
                    // Insert style attribute into the opening tag
                    if let Some(close_bracket) = trimmed.find('>') {
                        let tag = &trimmed[..close_bracket];
                        let rest = &trimmed[close_bracket..];
                        result.push_str(tag);
                        result.push_str(&format!(r#" style="text-align:{align}""#));
                        result.push_str(rest);
                        result.push('\n');
                        para_idx += 1;
                        continue;
                    }
                }
            }
            para_idx += 1;
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}
