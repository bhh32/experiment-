use dioxus::prelude::*;

use crate::editor::{get_cursor_position, set_cursor_position};

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

pub const LINE_HEIGHT_OPTIONS: &[(&str, &str)] = &[
    ("1.0", "Single"),
    ("1.15", "1.15"),
    ("1.5", "1.5"),
    ("2.0", "Double"),
    ("2.5", "2.5"),
    ("3.0", "Triple"),
];

#[component]
pub fn ToolbarUi(
    content: Signal<String>,
    font_family: Signal<String>,
    line_height: Signal<f32>,
) -> Element {
    rsx! {
        div { class: "toolbar",
            // Font selector
            select {
                class: "tool-select font-select",
                title: "Font Family",
                value: "{font_family}",
                onchange: move |evt| font_family.set(evt.value().clone()),
                for &f in FONT_OPTIONS {
                    option { value: f, "{f}" }
                }
            }

            // Line height selector
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
                    option { value: val, "{label}" }
                }
            }

            div { class: "tool-separator" }

            button {
                class: "tool-btn align-btn",
                title: "Align Left",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    // Remove any alignment prefix (default is left)
                    let (new_text, new_pos) = remove_alignment(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "L"
            }
            button {
                class: "tool-btn align-btn",
                title: "Center",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "center");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "C"
            }
            button {
                class: "tool-btn align-btn",
                title: "Align Right",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "right");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "R"
            }
            button {
                class: "tool-btn align-btn",
                title: "Justify",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_alignment(&val, cursor, "justify");
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "J"
            }

            div { class: "tool-separator" }

            button {
                class: "tool-btn",
                title: "Bold (Ctrl+B)",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::toggle_bold),
                "B"
            }
            button {
                class: "tool-btn",
                title: "Italic (Ctrl+I)",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::toggle_italic),
                "I"
            }
            button {
                class: "tool-btn",
                title: "Heading 1",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::insert_heading(&val, cursor, 1);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "H1"
            }
            button {
                class: "tool-btn",
                title: "Heading 2",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::insert_heading(&val, cursor, 2);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "H2"
            }
            button {
                class: "tool-btn",
                title: "Heading 3",
                onclick: move |_| {
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::insert_heading(&val, cursor, 3);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                },
                "H3"
            }
            button {
                class: "tool-btn",
                title: "Bullet List",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_list),
                "List"
            }
            button {
                class: "tool-btn",
                title: "Numbered List",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_ordered_list),
                "1."
            }
            button {
                class: "tool-btn",
                title: "Link (Ctrl+K)",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_link),
                "Link"
            }
            button {
                class: "tool-btn",
                title: "Code Block",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_code_block),
                "Code"
            }
            button {
                class: "tool-btn",
                title: "Horizontal Rule",
                onclick: move |_| apply_format(&mut content, toolbar::formatting::insert_hr),
                "HR"
            }
        }
    }
}
