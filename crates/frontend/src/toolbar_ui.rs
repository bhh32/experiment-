use dioxus::prelude::*;

use crate::editor::{get_cursor_position, set_cursor_position};

fn apply_format(content: &mut Signal<String>, f: fn(&str, usize) -> (String, usize)) {
    let val = content.read().clone();
    let cursor = get_cursor_position();
    let (new_text, new_pos) = f(&val, cursor);
    content.set(new_text);
    set_cursor_position(new_pos);
}

#[component]
pub fn ToolbarUi(content: Signal<String>) -> Element {
    rsx! {
        div { class: "toolbar",
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
