use dioxus::prelude::*;

use crate::editor::{get_cursor_position, set_cursor_position, get_selection_range};

fn remove_alignment(content: &str, cursor: usize) -> (String, usize) {
    let line_start = content[..cursor]
        .rfind('\n')
        .map(|p| p + 1)
        .unwrap_or(0);
    let line_end = content[line_start..]
        .find('\n')
        .map(|p| line_start + p)
        .unwrap_or(content.len());
    let line = &content[line_start..line_end];

    for prefix in &["{center}", "{right}", "{justify}"] {
        if line.starts_with(prefix) {
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..line_start]);
            result.push_str(&line[prefix.len()..]);
            result.push_str(&content[line_end..]);
            return (result, cursor.saturating_sub(prefix.len()).max(line_start));
        }
    }
    (content.to_string(), cursor)
}

fn apply_format(content: &mut Signal<String>, f: fn(&str, usize) -> (String, usize)) {
    let val = content.read().clone();
    let cursor = get_cursor_position();
    let (new_text, new_pos) = f(&val, cursor);
    content.set(new_text);
    set_cursor_position(new_pos);
}

/// Wrap the selected text (or insert at cursor) with the given marker.
fn apply_wrap_selection(content: &mut Signal<String>, marker: &str) {
    let val = content.read().clone();
    let (start, end) = get_selection_range();
    if start < end && end <= val.len() {
        // Wrap selection
        let selected = &val[start..end];
        let mut result = String::with_capacity(val.len() + marker.len() * 2);
        result.push_str(&val[..start]);
        result.push_str(marker);
        result.push_str(selected);
        result.push_str(marker);
        result.push_str(&val[end..]);
        content.set(result);
        set_cursor_position(end + marker.len() * 2);
    } else {
        // No selection — insert markers at cursor
        let cursor = start;
        let mut result = String::with_capacity(val.len() + marker.len() * 2);
        result.push_str(&val[..cursor]);
        result.push_str(marker);
        result.push_str(marker);
        result.push_str(&val[cursor..]);
        content.set(result);
        set_cursor_position(cursor + marker.len());
    }
}

pub const FONT_OPTIONS: &[&str] = &[
    "Calibri",
    "Arial",
    "Times New Roman",
    "Georgia",
    "Verdana",
    "Helvetica",
    "Courier New",
    "Garamond",
    "Palatino",
    "Cambria",
    "Trebuchet MS",
    "Tahoma",
    "Liberation Serif",
    "Liberation Sans",
];

pub const FONT_SIZE_OPTIONS: &[&str] = &[
    "8", "9", "10", "10.5", "11", "12", "14", "16", "18", "20", "24", "28", "36", "48", "72",
];

pub const LINE_HEIGHT_OPTIONS: &[(&str, &str)] = &[
    ("1.0", "Single"),
    ("1.15", "1.15"),
    ("1.5", "1.5"),
    ("2.0", "Double"),
    ("2.5", "2.5"),
    ("3.0", "Triple"),
];

pub const PARAGRAPH_STYLES: &[(&str, &str)] = &[
    ("body", "Body Text"),
    ("h1", "Heading 1"),
    ("h2", "Heading 2"),
    ("h3", "Heading 3"),
    ("h4", "Heading 4"),
    ("quote", "Quote"),
    ("code", "Code"),
];

#[component]
pub fn ToolbarUi(
    content: Signal<String>,
    font_family: Signal<String>,
    font_size: Signal<f32>,
    line_height: Signal<f32>,
) -> Element {
    rsx! {
        div { class: "toolbar",
            // Font family
            select {
                class: "tool-select font-select",
                title: "Font Family",
                value: "{font_family}",
                onchange: move |evt| font_family.set(evt.value().clone()),
                for &f in FONT_OPTIONS {
                    option {
                        value: f,
                        selected: *font_family.read() == f,
                        "{f}"
                    }
                }
            }

            // Font size
            select {
                class: "tool-select font-size-select",
                title: "Font Size",
                value: "{font_size}",
                onchange: move |evt| {
                    if let Ok(v) = evt.value().parse::<f32>() {
                        font_size.set(v);
                    }
                },
                for &s in FONT_SIZE_OPTIONS {
                    option {
                        value: s,
                        selected: format!("{}", *font_size.read()) == s || format!("{:.0}", *font_size.read()) == s,
                        "{s}"
                    }
                }
            }

            // Paragraph style
            select {
                class: "tool-select style-select",
                title: "Paragraph Style",
                onchange: move |evt| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = match evt.value().as_str() {
                        "h1" => toolbar::formatting::insert_heading(&val, cursor, 1),
                        "h2" => toolbar::formatting::insert_heading(&val, cursor, 2),
                        "h3" => toolbar::formatting::insert_heading(&val, cursor, 3),
                        "h4" => toolbar::formatting::insert_heading(&val, cursor, 4),
                        "quote" => {
                            let line_start = val[..cursor].rfind('\n').map(|p| p + 1).unwrap_or(0);
                            let mut result = String::with_capacity(val.len() + 2);
                            result.push_str(&val[..line_start]);
                            result.push_str("> ");
                            result.push_str(&val[line_start..]);
                            (result, cursor + 2)
                        }
                        "code" => toolbar::formatting::insert_code_block(&val, cursor),
                        _ => (val, cursor), // body text = no prefix
                    };
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                for &(val, label) in PARAGRAPH_STYLES {
                    option { value: val, "{label}" }
                }
            }

            div { class: "tool-separator" }

            // ─── Text Formatting ───
            button {
                class: "tool-btn fmt-bold",
                title: "Bold (Ctrl+B)",
                onclick: move |_| apply_wrap_selection(&mut content, "**"),
                span { class: "icon-bold", "B" }
            }
            button {
                class: "tool-btn fmt-italic",
                title: "Italic (Ctrl+I)",
                onclick: move |_| apply_wrap_selection(&mut content, "*"),
                span { class: "icon-italic", "I" }
            }
            button {
                class: "tool-btn fmt-underline",
                title: "Underline (Ctrl+U)",
                onclick: move |_| apply_wrap_selection(&mut content, "__"),
                span { class: "icon-underline", "U" }
            }
            button {
                class: "tool-btn fmt-strike",
                title: "Strikethrough",
                onclick: move |_| apply_wrap_selection(&mut content, "~~"),
                span { class: "icon-strike", "S" }
            }

            div { class: "tool-separator" }

            // ─── Alignment ───
            button {
                class: "tool-btn",
                title: "Align Left",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = remove_alignment(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                span { class: "icon-align", "☰" }
            }
            button {
                class: "tool-btn",
                title: "Center",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "center");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                span { class: "icon-center", "≡" }
            }
            button {
                class: "tool-btn",
                title: "Align Right",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "right");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                span { class: "icon-right", "☰" }
            }
            button {
                class: "tool-btn",
                title: "Justify",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "justify");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                span { class: "icon-justify", "☰" }
            }

            div { class: "tool-separator" }

            // ─── Lists ───
            button {
                class: "tool-btn",
                title: "Bullet List",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_list),
                span { class: "icon-ul", "•≡" }
            }
            button {
                class: "tool-btn",
                title: "Numbered List",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_ordered_list),
                span { class: "icon-ol", "1." }
            }

            div { class: "tool-separator" }

            // ─── Insert ───
            button {
                class: "tool-btn",
                title: "Link (Ctrl+K)",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_link),
                span { class: "icon-link", "\u{1F517}" }
            }
            button {
                class: "tool-btn",
                title: "Code Block",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_code_block),
                span { class: "icon-code", "</>" }
            }
            button {
                class: "tool-btn",
                title: "Horizontal Rule",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_hr),
                span { class: "icon-hr", "―" }
            }
            button {
                class: "tool-btn",
                title: "Page Break",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_page_break),
                span { class: "icon-pgbrk", "⊞" }
            }

            div { class: "tool-separator" }

            // Line spacing
            select {
                class: "tool-select line-height-select",
                title: "Line Spacing",
                value: "{line_height}",
                onchange: move |evt| {
                    if let Ok(v) = evt.value().parse::<f32>() {
                        line_height.set(v);
                    }
                },
                for &(val, label) in LINE_HEIGHT_OPTIONS {
                    option {
                        value: val,
                        selected: format!("{:.1}", *line_height.read()) == val || format!("{}", *line_height.read()) == val,
                        "{label}"
                    }
                }
            }
        }
    }
}
