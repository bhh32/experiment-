use comrak::{markdown_to_html as comrak_render, Options};

fn gfm_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.render.unsafe_ = false; // prevent raw HTML (XSS protection)
    opts
}

/// Render GFM markdown to sanitized HTML.
pub fn markdown_to_html(markdown: &str) -> String {
    comrak_render(markdown, &gfm_options())
}

/// Strip markdown to plain text (for DOCX paragraph content).
pub fn markdown_to_plain_text(markdown: &str) -> String {
    let html = markdown_to_html(markdown);
    // Simple HTML tag stripping
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
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading() {
        let html = markdown_to_html("# Hello World");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello World"));
    }

    #[test]
    fn renders_bold() {
        let html = markdown_to_html("**bold text**");
        assert!(html.contains("<strong>bold text</strong>"));
    }

    #[test]
    fn renders_gfm_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn blocks_raw_html() {
        let html = markdown_to_html("<script>alert('xss')</script>");
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn renders_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("<code"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn plain_text_strips_tags() {
        let text = markdown_to_plain_text("**bold** and *italic*");
        assert_eq!(text, "bold and italic");
    }

    #[test]
    fn renders_task_list() {
        let md = "- [x] Done\n- [ ] Todo";
        let html = markdown_to_html(md);
        assert!(html.contains("checked"));
    }
}
