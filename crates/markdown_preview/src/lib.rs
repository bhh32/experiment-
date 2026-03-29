use comrak::{markdown_to_html, Options};

fn gfm_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.render.unsafe_ = false;
    opts
}

/// Render markdown to HTML for the live preview pane.
pub fn render_preview(markdown: &str) -> String {
    markdown_to_html(markdown, &gfm_options())
}

/// Render markdown as if it were a DOCX document (Word-like styling).
pub fn render_docx_preview(markdown: &str) -> String {
    let html = render_preview(markdown);
    format!(
        r#"<div style="font-family: Calibri, sans-serif; font-size: 11pt; line-height: 1.15; max-width: 6.5in; margin: 1in auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1);">{html}</div>"#
    )
}

/// Render markdown as if it were an ODF document (LibreOffice-like styling).
pub fn render_odt_preview(markdown: &str) -> String {
    let html = render_preview(markdown);
    format!(
        r#"<div style="font-family: 'Liberation Serif', 'Times New Roman', serif; font-size: 12pt; line-height: 1.5; max-width: 6.5in; margin: 1in auto; padding: 1in; background: white; box-shadow: 0 0 10px rgba(0,0,0,0.1);">{html}</div>"#
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
