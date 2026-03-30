use dioxus::prelude::*;
use shared::DocStyle;

#[component]
pub fn PreviewPane(
    content: Signal<String>,
    mode: Signal<String>,
    font_family: Signal<String>,
    font_size: Signal<f32>,
    line_height: Signal<f32>,
) -> Element {
    let html = use_memo(move || {
        let text = content.read().clone();
        let m = mode.read().clone();
        let ff = font_family.read().clone();
        let fs = *font_size.read();
        let lh = *line_height.read();

        let mut ds = DocStyle::new(&ff, lh);
        ds.body_size_pt = fs;

        let processed = preprocess_markdown(&text);
        let mut base_html = markdown_preview::render_preview(&processed.clean_md);
        base_html = apply_alignments_by_content(&base_html, &processed.align_map);
        base_html = apply_page_breaks(&base_html, &processed.page_break_after);

        match m.as_str() {
            "docx" => render_docx_preview(&base_html, &ds),
            "odt" => render_odt_preview(&base_html, &ds),
            _ => render_md_preview(&base_html, &ds),
        }
    });

    rsx! {
        div {
            class: "preview-content",
            dangerous_inner_html: "{html}",
        }
    }
}

struct ProcessedMarkdown {
    clean_md: String,
    /// Map from stripped text content -> alignment (e.g. "My Title" -> "center")
    align_map: Vec<(String, String)>,
    /// Text content of paragraphs that should have a page break BEFORE them
    page_break_after: Vec<String>,
}

fn preprocess_markdown(markdown: &str) -> ProcessedMarkdown {
    let mut clean_lines: Vec<String> = Vec::new();
    let mut align_map: Vec<(String, String)> = Vec::new();
    let mut page_break_after: Vec<String> = Vec::new();
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
                clean_lines.push("\u{00a0}".to_string());
            } else {
                clean_lines.push(String::new());
            }
            continue;
        }

        // Record alignment for this line's text content
        if align != "left" {
            // Strip markdown heading prefix to get the text that will appear in HTML
            let display_text = text.trim().trim_start_matches('#').trim().to_string();
            if !display_text.is_empty() {
                align_map.push((display_text, align.to_string()));
            }
        }

        // Record page break before this content
        if next_has_page_break {
            let display_text = text.trim().trim_start_matches('#').trim().to_string();
            if !display_text.is_empty() {
                page_break_after.push(display_text);
            }
            next_has_page_break = false;
        }

        consecutive_blanks = 0;
        clean_lines.push(text.to_string());
    }

    ProcessedMarkdown {
        clean_md: clean_lines.join("\n"),
        align_map,
        page_break_after,
    }
}

/// Apply alignments by finding block elements whose text content matches.
fn apply_alignments_by_content(html: &str, align_map: &[(String, String)]) -> String {
    if align_map.is_empty() {
        return html.to_string();
    }

    let mut result = String::with_capacity(html.len() + align_map.len() * 40);

    for line in html.lines() {
        let trimmed = line.trim();
        let mut matched = false;

        // Check if this is a block-level opening tag
        if trimmed.starts_with("<p>")
            || trimmed.starts_with("<h1>")
            || trimmed.starts_with("<h2>")
            || trimmed.starts_with("<h3>")
            || trimmed.starts_with("<h4>")
            || trimmed.starts_with("<h5>")
            || trimmed.starts_with("<h6>")
        {
            // Extract text content (strip HTML tags)
            let text_content = strip_tags(trimmed);
            let text_trimmed = text_content.trim();

            for (content, align) in align_map {
                if text_trimmed.contains(content.as_str()) {
                    // Insert style attribute into the opening tag
                    if let Some(close_bracket) = trimmed.find('>') {
                        let tag = &trimmed[..close_bracket];
                        let rest = &trimmed[close_bracket..];
                        result.push_str(tag);
                        result.push_str(&format!(r#" style="text-align:{align}""#));
                        result.push_str(rest);
                        result.push('\n');
                        matched = true;
                        break;
                    }
                }
            }
        }

        if !matched {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Apply page breaks by inserting a page-break div before matching block elements.
fn apply_page_breaks(html: &str, page_break_after: &[String]) -> String {
    if page_break_after.is_empty() {
        return html.to_string();
    }

    let block_starts = ["<p", "<h1", "<h2", "<h3", "<h4", "<h5", "<h6",
                        "<ul", "<ol", "<table", "<pre", "<blockquote"];
    let mut result = String::with_capacity(html.len() + page_break_after.len() * 50);

    for line in html.lines() {
        let trimmed = line.trim();
        let is_block = block_starts.iter().any(|tag| trimmed.starts_with(tag));

        if is_block {
            let text_content = strip_tags(trimmed);
            let text_trimmed = text_content.trim();

            for content in page_break_after {
                if text_trimmed.contains(content.as_str()) {
                    result.push_str(r#"<div class="page-break"></div>"#);
                    result.push('\n');
                    break;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Strip HTML tags from a string, returning plain text content.
fn strip_tags(html: &str) -> String {
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

/// DOCX preview
fn render_docx_preview(html: &str, ds: &DocStyle) -> String {
    let f = &ds.body_font;
    let body = ds.body_size_pt;
    let h1 = ds.heading1_pt();
    let h2 = ds.heading2_pt();
    let h3 = ds.heading3_pt();
    let h4 = ds.heading4_pt();
    let code = ds.code_size_pt();
    let cf = &ds.code_font;
    let lh = ds.line_spacing;
    let pa = ds.para_after_pt();
    let hb = ds.heading_before_pt();
    let ha = ds.heading_after_pt();

    format!(
        r##"<div class="docx-preview" style="font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh};">
<style>
.docx-preview * {{ box-sizing: border-box; }}
.docx-preview {{ display: flex; flex-direction: column; align-items: center; gap: 20px; padding: 20px 0; }}
.docx-page {{ width: 8.5in; min-height: 11in; padding: 1in; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.15); position: relative; }}
.docx-preview h1 {{ font-family: '{f}', sans-serif; font-size: {h1:.1}pt; font-weight: bold; line-height: 1.15; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h2 {{ font-family: '{f}', sans-serif; font-size: {h2:.1}pt; font-weight: bold; line-height: 1.15; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h3 {{ font-family: '{f}', sans-serif; font-size: {h3:.1}pt; font-weight: normal; line-height: {lh}; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h4 {{ font-family: '{f}', sans-serif; font-size: {h4:.1}pt; font-weight: normal; line-height: {lh}; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview p {{ font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 {pa}pt 0; }}
.docx-preview ul, .docx-preview ol {{ font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 {pa}pt 0; padding-left: 0.5in; }}
.docx-preview ul {{ list-style-type: disc; }}
.docx-preview ul ul {{ list-style-type: circle; }}
.docx-preview ul ul ul {{ list-style-type: square; }}
.docx-preview ol {{ list-style-type: decimal; }}
.docx-preview ol ol {{ list-style-type: lower-alpha; }}
.docx-preview ol ol ol {{ list-style-type: lower-roman; }}
.docx-preview li {{ margin: 0 0 2pt 0; }}
.docx-preview blockquote {{ margin: 0 0 {pa}pt 0.5in; padding: 0; border: none; font-style: italic; color: #555555; }}
.docx-preview blockquote p {{ font-style: italic; color: #555555; font-family: '{f}', sans-serif; font-size: {body}pt; }}
.docx-preview pre {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; line-height: 1.0; margin: 0 0 {pa}pt 0; padding: 6pt 0.25in; background: none; border: none; }}
.docx-preview pre code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #333333; background: none; padding: 0; }}
.docx-preview code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #c7254e; background: #f0f0f0; padding: 1pt 3pt; border-radius: 2pt; }}
.docx-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 {pa}pt 0; font-family: '{f}', sans-serif; font-size: {body}pt; }}
.docx-preview th {{ border: 1pt solid #999999; padding: 2pt 6pt; background: #e8e8e8; font-weight: bold; text-align: left; vertical-align: middle; }}
.docx-preview td {{ border: 1pt solid #cccccc; padding: 2pt 6pt; text-align: left; vertical-align: middle; }}
.docx-preview hr {{ border: none; text-align: center; color: #cccccc; margin: 12pt 0; }}
.docx-preview a {{ color: #0563c1; text-decoration: underline; }}
.docx-preview strong {{ font-weight: bold; }}
.docx-preview em {{ font-style: italic; }}
.docx-preview del {{ text-decoration: line-through; }}
.docx-preview input[type="checkbox"] {{ margin-right: 4pt; }}
.page-break {{ break-after: always; height: 0; margin: 0; padding: 0; border: none; }}
</style>
{pages}</div>"##,
        pages = split_into_pages(html, "docx-page")
    )
}

/// ODF preview
fn render_odt_preview(html: &str, ds: &DocStyle) -> String {
    let f = if ds.body_font == "Calibri" { "Liberation Serif" } else { &ds.body_font };
    let hf = if ds.body_font == "Calibri" { "Liberation Sans" } else { &ds.body_font };
    let cf = "Liberation Mono";
    let body = 12.0f32;
    let lh = ds.line_spacing;
    let h1 = body * 2.0;
    let h2 = body * 1.5;
    let h3 = body * 1.17;
    let code = body * 0.83;

    format!(
        r##"<div class="odt-preview" style="font-family: '{f}', 'Times New Roman', serif; font-size: {body}pt; line-height: {lh};">
<style>
.odt-preview {{ display: flex; flex-direction: column; align-items: center; gap: 20px; padding: 20px 0; }}
.odt-page {{ width: 8.5in; min-height: 11in; padding: 1in; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }}
.odt-preview h1 {{ font-family: '{hf}', 'Arial', sans-serif; font-size: {h1:.1}pt; font-weight: bold; margin: 14pt 0 7pt 0; border: none; padding: 0; }}
.odt-preview h2 {{ font-family: '{hf}', 'Arial', sans-serif; font-size: {h2:.1}pt; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; }}
.odt-preview h3 {{ font-family: '{hf}', 'Arial', sans-serif; font-size: {h3:.1}pt; font-weight: bold; margin: 10pt 0 5pt 0; border: none; padding: 0; }}
.odt-preview p {{ font-family: '{f}', 'Times New Roman', serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 10pt 0; }}
.odt-preview ul, .odt-preview ol {{ font-family: '{f}', serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 10pt 0; padding-left: 0.5in; }}
.odt-preview li {{ margin: 0 0 3pt 0; }}
.odt-preview blockquote {{ margin: 0 0 10pt 0.5in; font-style: italic; color: #555; }}
.odt-preview pre {{ font-family: '{cf}', 'Courier New', monospace; font-size: {code:.1}pt; line-height: 1.2; margin: 0 0 10pt 0; padding: 8pt 0.25in; background: #f8f8f8; border: 1pt solid #ddd; }}
.odt-preview pre code {{ font-family: '{cf}', 'Courier New', monospace; font-size: {code:.1}pt; color: #333; background: none; padding: 0; }}
.odt-preview code {{ font-family: '{cf}', 'Courier New', monospace; font-size: {code:.1}pt; color: #333; background: #f0f0f0; padding: 1pt 3pt; }}
.odt-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 10pt 0; font-family: '{f}', serif; font-size: {body}pt; }}
.odt-preview th {{ border: 1pt solid #000; padding: 4pt 6pt; background: #e8e8e8; font-weight: bold; }}
.odt-preview td {{ border: 1pt solid #000; padding: 4pt 6pt; }}
.odt-preview hr {{ border: none; border-bottom: 1pt solid #000; margin: 12pt 0; }}
.odt-preview a {{ color: #0000ee; text-decoration: underline; }}
.odt-preview strong {{ font-weight: bold; }}
.odt-preview em {{ font-style: italic; }}
.odt-preview del {{ text-decoration: line-through; }}
.page-break {{ break-after: always; height: 0; margin: 0; padding: 0; border: none; }}
</style>
{pages}</div>"##,
        pages = split_into_pages(html, "odt-page")
    )
}

/// Plain markdown preview — also splits at page breaks for consistency.
fn render_md_preview(html: &str, ds: &DocStyle) -> String {
    let f = &ds.body_font;
    let body = ds.body_size_pt;
    let lh = ds.line_spacing;

    format!(
        r##"<div class="md-preview" style="font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh};">
<style>
.md-preview {{ display: flex; flex-direction: column; align-items: center; gap: 20px; padding: 20px 0; }}
.md-page {{ width: 8.5in; min-height: 11in; padding: 1in; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }}
</style>
{pages}</div>"##,
        pages = split_into_pages(html, "md-page")
    )
}

/// Split HTML content at page-break markers into separate page divs.
fn split_into_pages(html: &str, page_class: &str) -> String {
    let marker = r#"<div class="page-break"></div>"#;
    let pages: Vec<&str> = html.split(marker).collect();

    if pages.len() <= 1 {
        return format!(r#"<div class="{page_class}">{html}</div>"#);
    }

    pages
        .iter()
        .map(|page| format!(r#"<div class="{page_class}">{page}</div>"#))
        .collect::<Vec<_>>()
        .join("\n")
}
