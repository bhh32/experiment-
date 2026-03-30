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

        // Parse markdown → Document IR → HTML (single source of truth)
        let doc = document_ir::parse_markdown(&text);
        let base_html = document_ir::render_to_html(&doc, &ds);

        match m.as_str() {
            "docx" => wrap_docx_preview(&base_html, &ds),
            "odt" => wrap_odt_preview(&base_html, &ds),
            _ => wrap_md_preview(&base_html, &ds),
        }
    });

    rsx! {
        div {
            class: "preview-content",
            dangerous_inner_html: "{html}",
        }
    }
}

/// Split HTML at page-break markers into separate page divs.
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

/// DOCX preview wrapper — Word-like styling around the IR-generated HTML.
fn wrap_docx_preview(html: &str, ds: &DocStyle) -> String {
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
.docx-page {{ width: 8.5in; min-height: 11in; padding: 1in; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }}
.docx-preview h1 {{ font-family: '{f}', sans-serif; font-size: {h1:.1}pt; font-weight: bold; line-height: 1.15; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h2 {{ font-family: '{f}', sans-serif; font-size: {h2:.1}pt; font-weight: bold; line-height: 1.15; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h3 {{ font-family: '{f}', sans-serif; font-size: {h3:.1}pt; font-weight: normal; line-height: {lh}; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h4 {{ font-family: '{f}', sans-serif; font-size: {h4:.1}pt; font-weight: normal; line-height: {lh}; margin: {hb}pt 0 {ha}pt 0; border: none; padding: 0; color: #000; }}
.docx-preview p {{ font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 {pa}pt 0; }}
.docx-preview ul, .docx-preview ol {{ font-family: '{f}', sans-serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 {pa}pt 0; padding-left: 0.5in; }}
.docx-preview ul {{ list-style-type: disc; }} .docx-preview ul ul {{ list-style-type: circle; }} .docx-preview ul ul ul {{ list-style-type: square; }}
.docx-preview ol {{ list-style-type: decimal; }} .docx-preview ol ol {{ list-style-type: lower-alpha; }}
.docx-preview li {{ margin: 0 0 2pt 0; }}
.docx-preview blockquote {{ margin: 0 0 {pa}pt 0.5in; font-style: italic; color: #555; }}
.docx-preview pre {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; line-height: 1.0; margin: 0 0 {pa}pt 0; padding: 6pt 0.25in; }}
.docx-preview pre code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #333; background: none; padding: 0; }}
.docx-preview code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #c7254e; background: #f0f0f0; padding: 1pt 3pt; border-radius: 2pt; }}
.docx-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 {pa}pt 0; font-family: '{f}', sans-serif; font-size: {body}pt; }}
.docx-preview th {{ border: 1pt solid #999; padding: 2pt 6pt; background: #e8e8e8; font-weight: bold; text-align: left; }}
.docx-preview td {{ border: 1pt solid #ccc; padding: 2pt 6pt; text-align: left; }}
.docx-preview hr {{ border: none; border-bottom: 1pt solid #ccc; margin: 12pt 0; }}
.docx-preview a {{ color: #0563c1; text-decoration: underline; }}
.docx-preview strong {{ font-weight: bold; }}
.docx-preview em {{ font-style: italic; }}
.docx-preview del {{ text-decoration: line-through; }}
.docx-preview input[type="checkbox"] {{ margin-right: 4pt; }}
</style>
{pages}</div>"##,
        pages = split_into_pages(html, "docx-page")
    )
}

/// ODF preview wrapper — LibreOffice-like styling.
fn wrap_odt_preview(html: &str, ds: &DocStyle) -> String {
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
.odt-preview h1 {{ font-family: '{hf}', sans-serif; font-size: {h1:.1}pt; font-weight: bold; margin: 14pt 0 7pt 0; border: none; padding: 0; }}
.odt-preview h2 {{ font-family: '{hf}', sans-serif; font-size: {h2:.1}pt; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; }}
.odt-preview h3 {{ font-family: '{hf}', sans-serif; font-size: {h3:.1}pt; font-weight: bold; margin: 10pt 0 5pt 0; border: none; padding: 0; }}
.odt-preview p {{ font-family: '{f}', serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 10pt 0; }}
.odt-preview ul, .odt-preview ol {{ font-family: '{f}', serif; font-size: {body}pt; line-height: {lh}; margin: 0 0 10pt 0; padding-left: 0.5in; }}
.odt-preview li {{ margin: 0 0 3pt 0; }}
.odt-preview blockquote {{ margin: 0 0 10pt 0.5in; font-style: italic; color: #555; }}
.odt-preview pre {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; line-height: 1.2; margin: 0 0 10pt 0; padding: 8pt 0.25in; background: #f8f8f8; border: 1pt solid #ddd; }}
.odt-preview pre code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #333; background: none; padding: 0; }}
.odt-preview code {{ font-family: '{cf}', monospace; font-size: {code:.1}pt; color: #333; background: #f0f0f0; padding: 1pt 3pt; }}
.odt-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 10pt 0; font-family: '{f}', serif; font-size: {body}pt; }}
.odt-preview th {{ border: 1pt solid #000; padding: 4pt 6pt; background: #e8e8e8; font-weight: bold; }}
.odt-preview td {{ border: 1pt solid #000; padding: 4pt 6pt; }}
.odt-preview hr {{ border: none; border-bottom: 1pt solid #000; margin: 12pt 0; }}
.odt-preview a {{ color: #0000ee; text-decoration: underline; }}
.odt-preview strong {{ font-weight: bold; }}
.odt-preview em {{ font-style: italic; }}
.odt-preview del {{ text-decoration: line-through; }}
</style>
{pages}</div>"##,
        pages = split_into_pages(html, "odt-page")
    )
}

/// Plain markdown/print preview wrapper.
fn wrap_md_preview(html: &str, ds: &DocStyle) -> String {
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
