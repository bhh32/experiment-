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
        let (clean_md, alignments) = preprocess_alignments(&text);
        let mut base_html = markdown_preview::render_preview(&clean_md);
        base_html = apply_alignments(&base_html, &alignments);

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

/// DOCX preview — uses the exact same DocStyle values as the DOCX export.
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
        r##"<div class="docx-preview" style="font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; width: 6.5in; margin: 0 auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1); min-height: 11in;">
<style>
.docx-preview * {{ box-sizing: border-box; }}
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
</style>
{html}</div>"##
    )
}

/// ODF preview — Liberation fonts, 12pt body, LibreOffice-style spacing.
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
        r##"<div class="odt-preview" style="font-family: '{f}', 'Times New Roman', serif; font-size: {body}pt; line-height: {lh}; width: 6.5in; margin: 0 auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1); min-height: 11in;">
<style>
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
</style>
{html}</div>"##
    )
}

/// Plain markdown preview — uses user's font but web-native styling.
fn render_md_preview(html: &str, ds: &DocStyle) -> String {
    let f = &ds.body_font;
    let body = ds.body_size_pt;
    let lh = ds.line_spacing;

    format!(
        r##"<div class="md-preview" style="font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; padding: 20px;">
{html}</div>"##
    )
}

fn preprocess_alignments(markdown: &str) -> (String, Vec<String>) {
    let mut clean_lines = Vec::new();
    let mut alignments = Vec::new();
    let mut current_align = "left".to_string();

    for line in markdown.lines() {
        let (align, text) = toolbar::formatting::parse_alignment(line);
        if align != "left" {
            current_align = align.to_string();
        }
        if !text.trim().is_empty() && (clean_lines.is_empty() || clean_lines.last() == Some(&String::new())) {
            alignments.push(current_align.clone());
        }
        if text.trim().is_empty() {
            current_align = "left".to_string();
        }
        clean_lines.push(text.to_string());
    }

    (clean_lines.join("\n"), alignments)
}

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
