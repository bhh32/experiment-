/// Pure string manipulation functions for markdown formatting.
/// These take the editor content, cursor position, and return
/// the new content and new cursor position.

pub mod formatting {
    /// Wrap selected text (or insert at cursor) with bold markers.
    pub fn toggle_bold(content: &str, cursor: usize) -> (String, usize) {
        wrap_markers(content, cursor, "**")
    }

    /// Wrap selected text (or insert at cursor) with italic markers.
    pub fn toggle_italic(content: &str, cursor: usize) -> (String, usize) {
        wrap_markers(content, cursor, "*")
    }

    /// Insert a heading at the current line.
    pub fn insert_heading(content: &str, cursor: usize, level: u8) -> (String, usize) {
        let prefix = format!("{} ", "#".repeat(level as usize));
        insert_line_prefix(content, cursor, &prefix)
    }

    /// Insert an unordered list marker at the current line.
    pub fn insert_list(content: &str, cursor: usize) -> (String, usize) {
        insert_line_prefix(content, cursor, "- ")
    }

    /// Insert an ordered list marker at the current line.
    pub fn insert_ordered_list(content: &str, cursor: usize) -> (String, usize) {
        insert_line_prefix(content, cursor, "1. ")
    }

    /// Insert a link template at cursor.
    pub fn insert_link(content: &str, cursor: usize) -> (String, usize) {
        let insertion = "[link text](url)";
        let mut result = String::with_capacity(content.len() + insertion.len());
        result.push_str(&content[..cursor]);
        result.push_str(insertion);
        result.push_str(&content[cursor..]);
        // Place cursor at "link text" for easy editing
        (result, cursor + 1)
    }

    /// Insert a code block at cursor.
    pub fn insert_code_block(content: &str, cursor: usize) -> (String, usize) {
        let block = "```\n\n```";
        let mut result = String::with_capacity(content.len() + block.len() + 2);

        // Add newline before if not at start of line
        let needs_newline = cursor > 0 && content.as_bytes().get(cursor - 1) != Some(&b'\n');

        result.push_str(&content[..cursor]);
        if needs_newline {
            result.push('\n');
        }
        result.push_str(block);
        result.push_str(&content[cursor..]);

        let new_cursor = if needs_newline {
            cursor + 5 // \n``` + \n
        } else {
            cursor + 4 // ``` + \n
        };
        (result, new_cursor)
    }

    /// Insert a horizontal rule at cursor.
    pub fn insert_hr(content: &str, cursor: usize) -> (String, usize) {
        let hr = "\n---\n";
        let mut result = String::with_capacity(content.len() + hr.len());
        result.push_str(&content[..cursor]);
        result.push_str(hr);
        result.push_str(&content[cursor..]);
        (result, cursor + hr.len())
    }

    fn wrap_markers(content: &str, cursor: usize, marker: &str) -> (String, usize) {
        let mut result = String::with_capacity(content.len() + marker.len() * 2);
        result.push_str(&content[..cursor]);
        result.push_str(marker);
        result.push_str(marker);
        result.push_str(&content[cursor..]);
        (result, cursor + marker.len())
    }

    /// Alignment prefixes recognized by the preview and export pipeline.
    const ALIGN_PREFIXES: &[&str] = &["{center}", "{right}", "{justify}"];

    /// Toggle alignment on the current line. If the line already has this
    /// alignment, remove it. If it has a different alignment, replace it.
    pub fn toggle_alignment(content: &str, cursor: usize, alignment: &str) -> (String, usize) {
        let line_start = content[..cursor]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);

        let line_end = content[line_start..]
            .find('\n')
            .map(|p| line_start + p)
            .unwrap_or(content.len());

        let line = &content[line_start..line_end];
        let prefix = format!("{{{alignment}}}");

        // Check if line already has this alignment
        if line.starts_with(&prefix) {
            // Remove it (toggle off)
            let stripped = &line[prefix.len()..];
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..line_start]);
            result.push_str(stripped);
            result.push_str(&content[line_end..]);
            let new_cursor = cursor.saturating_sub(prefix.len());
            return (result, new_cursor.max(line_start));
        }

        // Remove any existing alignment prefix
        let mut clean_line = line;
        let mut removed_len = 0;
        for &ap in ALIGN_PREFIXES {
            if line.starts_with(ap) {
                clean_line = &line[ap.len()..];
                removed_len = ap.len();
                break;
            }
        }

        // Add the new prefix
        let mut result = String::with_capacity(content.len() + prefix.len());
        result.push_str(&content[..line_start]);
        result.push_str(&prefix);
        result.push_str(clean_line);
        result.push_str(&content[line_end..]);
        let new_cursor = cursor - removed_len + prefix.len();
        (result, new_cursor)
    }

    /// Strip alignment prefix from a line, returning (alignment, clean_text).
    pub fn parse_alignment(line: &str) -> (&str, &str) {
        for &prefix in ALIGN_PREFIXES {
            if line.starts_with(prefix) {
                let align = &prefix[1..prefix.len() - 1]; // strip { }
                return (align, &line[prefix.len()..]);
            }
        }
        ("left", line)
    }

    fn insert_line_prefix(content: &str, cursor: usize, prefix: &str) -> (String, usize) {
        // Find start of current line
        let line_start = content[..cursor]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);

        let mut result = String::with_capacity(content.len() + prefix.len());
        result.push_str(&content[..line_start]);
        result.push_str(prefix);
        result.push_str(&content[line_start..]);
        (result, cursor + prefix.len())
    }
}

#[cfg(test)]
mod tests {
    use super::formatting::*;

    #[test]
    fn bold_empty() {
        let (text, pos) = toggle_bold("", 0);
        assert_eq!(text, "****");
        assert_eq!(pos, 2);
    }

    #[test]
    fn bold_at_cursor() {
        let (text, pos) = toggle_bold("hello world", 5);
        assert_eq!(text, "hello**** world");
        assert_eq!(pos, 7);
    }

    #[test]
    fn italic_empty() {
        let (text, pos) = toggle_italic("", 0);
        assert_eq!(text, "**");
        assert_eq!(pos, 1);
    }

    #[test]
    fn heading_level_1() {
        let (text, _) = insert_heading("hello", 0, 1);
        assert_eq!(text, "# hello");
    }

    #[test]
    fn heading_level_2() {
        let (text, _) = insert_heading("hello", 0, 2);
        assert_eq!(text, "## hello");
    }

    #[test]
    fn heading_level_3() {
        let (text, _) = insert_heading("hello", 0, 3);
        assert_eq!(text, "### hello");
    }

    #[test]
    fn unordered_list() {
        let (text, _) = insert_list("item", 0);
        assert_eq!(text, "- item");
    }

    #[test]
    fn ordered_list() {
        let (text, _) = insert_ordered_list("item", 0);
        assert_eq!(text, "1. item");
    }

    #[test]
    fn link_insertion() {
        let (text, pos) = insert_link("", 0);
        assert_eq!(text, "[link text](url)");
        assert_eq!(pos, 1); // cursor inside []
    }

    #[test]
    fn code_block_at_start() {
        let (text, pos) = insert_code_block("", 0);
        assert_eq!(text, "```\n\n```");
        assert_eq!(pos, 4); // cursor between the fences
    }

    #[test]
    fn hr_insertion() {
        let (text, _) = insert_hr("above", 5);
        assert!(text.contains("---"));
        assert!(text.starts_with("above"));
    }

    #[test]
    fn center_adds_prefix() {
        let (text, _) = toggle_alignment("hello", 0, "center");
        assert_eq!(text, "{center}hello");
    }

    #[test]
    fn center_toggles_off() {
        let (text, _) = toggle_alignment("{center}hello", 8, "center");
        assert_eq!(text, "hello");
    }

    #[test]
    fn center_replaces_right() {
        let (text, _) = toggle_alignment("{right}hello", 7, "center");
        assert_eq!(text, "{center}hello");
    }

    #[test]
    fn right_alignment() {
        let (text, _) = toggle_alignment("hello", 0, "right");
        assert_eq!(text, "{right}hello");
    }

    #[test]
    fn parse_center() {
        let (align, text) = parse_alignment("{center}hello world");
        assert_eq!(align, "center");
        assert_eq!(text, "hello world");
    }

    #[test]
    fn parse_no_alignment() {
        let (align, text) = parse_alignment("hello world");
        assert_eq!(align, "left");
        assert_eq!(text, "hello world");
    }

    #[test]
    fn justify_alignment() {
        let (text, _) = toggle_alignment("hello", 0, "justify");
        assert_eq!(text, "{justify}hello");
    }
}
