use comrak::{markdown_to_html, Options};

fn gfm_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.render.unsafe_ = false;
    opts.render.hardbreaks = true;
    opts
}

/// Render markdown to HTML for the live preview pane.
pub fn render_preview(markdown: &str) -> String {
    markdown_to_html(markdown, &gfm_options())
}

/// Render markdown as if it were a DOCX document (Word-like styling).
/// Matches the formatting produced by markdown_to_docx in the conversion crate.
pub fn render_docx_preview(markdown: &str) -> String {
    let html = render_preview(markdown);
    format!(
        r#"<div class="docx-preview" style="font-family: Calibri, sans-serif; font-size: 11pt; line-height: 1.15; max-width: 6.5in; margin: 0 auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1);">
<style>
.docx-preview h1 {{ font-family: Calibri, sans-serif; font-size: 26pt; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h2 {{ font-family: Calibri, sans-serif; font-size: 20pt; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h3 {{ font-family: Calibri, sans-serif; font-size: 14pt; font-weight: normal; margin: 12pt 0 6pt 0; border: none; padding: 0; color: #000; }}
.docx-preview h4 {{ font-family: Calibri, sans-serif; font-size: 12pt; font-weight: normal; margin: 12pt 0 6pt 0; border: none; padding: 0; color: #000; }}
.docx-preview p {{ font-family: Calibri, sans-serif; font-size: 11pt; line-height: 1.15; margin: 0 0 8pt 0; }}
.docx-preview ul, .docx-preview ol {{ font-family: Calibri, sans-serif; font-size: 11pt; line-height: 1.15; margin: 0 0 8pt 0; padding-left: 0.5in; }}
.docx-preview ul {{ list-style-type: disc; }}
.docx-preview ul ul {{ list-style-type: circle; }}
.docx-preview ul ul ul {{ list-style-type: square; }}
.docx-preview ol {{ list-style-type: decimal; }}
.docx-preview ol ol {{ list-style-type: lower-alpha; }}
.docx-preview ol ol ol {{ list-style-type: lower-roman; }}
.docx-preview li {{ margin: 0 0 2pt 0; }}
.docx-preview blockquote {{ margin: 0 0 8pt 0.5in; padding: 0; border: none; font-style: italic; color: #555; }}
.docx-preview blockquote p {{ font-style: italic; color: #555; }}
.docx-preview pre {{ font-family: Consolas, monospace; font-size: 10pt; line-height: 1.0; margin: 0 0 8pt 0; padding: 6pt 0.25in; background: none; border: none; border-left: 2pt solid #ccc; border-right: 2pt solid #ccc; }}
.docx-preview pre code {{ font-family: Consolas, monospace; font-size: 10pt; color: #333; background: none; padding: 0; }}
.docx-preview code {{ font-family: Consolas, monospace; font-size: 10pt; color: #c7254e; background: #f0f0f0; padding: 1pt 3pt; border-radius: 2pt; }}
.docx-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 8pt 0; font-family: Calibri, sans-serif; font-size: 11pt; }}
.docx-preview th {{ border: 1pt solid #999; padding: 2pt 6pt; background: #e8e8e8; font-weight: bold; text-align: left; vertical-align: middle; }}
.docx-preview td {{ border: 1pt solid #ccc; padding: 2pt 6pt; text-align: left; vertical-align: middle; }}
.docx-preview hr {{ border: none; border-bottom: 1pt solid #ccc; margin: 12pt 0; }}
.docx-preview a {{ color: #0563c1; text-decoration: underline; }}
.docx-preview strong {{ font-weight: bold; }}
.docx-preview em {{ font-style: italic; }}
.docx-preview del {{ text-decoration: line-through; }}
.docx-preview img {{ max-width: 100%; }}
.docx-preview input[type="checkbox"] {{ margin-right: 4pt; }}
</style>
{html}</div>"#
    )
}

/// Render markdown as if it were an ODF document (LibreOffice-like styling).
pub fn render_odt_preview(markdown: &str) -> String {
    let html = render_preview(markdown);
    format!(
        r#"<div class="odt-preview" style="font-family: 'Liberation Serif', 'Times New Roman', serif; font-size: 12pt; line-height: 1.5; max-width: 6.5in; margin: 0 auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1);">
<style>
.odt-preview h1 {{ font-family: 'Liberation Sans', 'Arial', sans-serif; font-size: 24pt; font-weight: bold; margin: 14pt 0 7pt 0; border: none; padding: 0; color: #000; }}
.odt-preview h2 {{ font-family: 'Liberation Sans', 'Arial', sans-serif; font-size: 18pt; font-weight: bold; margin: 12pt 0 6pt 0; border: none; padding: 0; color: #000; }}
.odt-preview h3 {{ font-family: 'Liberation Sans', 'Arial', sans-serif; font-size: 14pt; font-weight: bold; margin: 10pt 0 5pt 0; border: none; padding: 0; color: #000; }}
.odt-preview h4 {{ font-family: 'Liberation Sans', 'Arial', sans-serif; font-size: 12pt; font-weight: bold; font-style: italic; margin: 10pt 0 5pt 0; border: none; padding: 0; color: #000; }}
.odt-preview p {{ font-family: 'Liberation Serif', 'Times New Roman', serif; font-size: 12pt; line-height: 1.5; margin: 0 0 10pt 0; }}
.odt-preview ul, .odt-preview ol {{ font-family: 'Liberation Serif', 'Times New Roman', serif; font-size: 12pt; line-height: 1.5; margin: 0 0 10pt 0; padding-left: 0.5in; }}
.odt-preview li {{ margin: 0 0 3pt 0; }}
.odt-preview blockquote {{ margin: 0 0 10pt 0.5in; padding: 0; border: none; font-style: italic; color: #555; }}
.odt-preview blockquote p {{ font-style: italic; color: #555; }}
.odt-preview pre {{ font-family: 'Liberation Mono', 'Courier New', monospace; font-size: 10pt; line-height: 1.2; margin: 0 0 10pt 0; padding: 8pt 0.25in; background: #f8f8f8; border: 1pt solid #ddd; }}
.odt-preview pre code {{ font-family: 'Liberation Mono', 'Courier New', monospace; font-size: 10pt; color: #333; background: none; padding: 0; }}
.odt-preview code {{ font-family: 'Liberation Mono', 'Courier New', monospace; font-size: 10pt; color: #333; background: #f0f0f0; padding: 1pt 3pt; }}
.odt-preview table {{ border-collapse: collapse; width: 100%; margin: 0 0 10pt 0; font-family: 'Liberation Serif', 'Times New Roman', serif; font-size: 12pt; }}
.odt-preview th {{ border: 1pt solid #000; padding: 4pt 6pt; background: #e8e8e8; font-weight: bold; text-align: left; }}
.odt-preview td {{ border: 1pt solid #000; padding: 4pt 6pt; text-align: left; }}
.odt-preview hr {{ border: none; border-bottom: 1pt solid #000; margin: 12pt 0; }}
.odt-preview a {{ color: #0000ee; text-decoration: underline; }}
.odt-preview strong {{ font-weight: bold; }}
.odt-preview em {{ font-style: italic; }}
.odt-preview del {{ text-decoration: line-through; }}
</style>
{html}</div>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading() {
        let html = render_preview("# Hello");
        assert!(html.contains("<h1>"));
    }

    #[test]
    fn renders_bold() {
        let html = render_preview("**bold**");
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn renders_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_preview(md);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn blocks_raw_html() {
        let html = render_preview("<script>alert('xss')</script>");
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn docx_preview_has_calibri() {
        let html = render_docx_preview("test");
        assert!(html.contains("Calibri"));
    }

    #[test]
    fn odt_preview_has_liberation_serif() {
        let html = render_odt_preview("test");
        assert!(html.contains("Liberation Serif"));
    }

    #[test]
    fn renders_code_block() {
        let md = "```\ncode here\n```";
        let html = render_preview(md);
        assert!(html.contains("<code>"));
    }
}
