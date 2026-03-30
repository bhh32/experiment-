use dioxus::prelude::*;

use crate::editor::{get_cursor_position, set_cursor_position};

struct StyleDef {
    label: &'static str,
    size: &'static str,
    prefix: &'static str,
}

const STYLES: &[StyleDef] = &[
    StyleDef { label: "Heading 1", size: "26pt", prefix: "# " },
    StyleDef { label: "Heading 2", size: "20pt", prefix: "## " },
    StyleDef { label: "Heading 3", size: "14pt", prefix: "### " },
    StyleDef { label: "Body Text", size: "11pt", prefix: "" },
    StyleDef { label: "Caption", size: "10pt", prefix: "" },
    StyleDef { label: "Code", size: "10pt", prefix: "```\n" },
    StyleDef { label: "Quote", size: "11pt", prefix: "> " },
];

#[component]
pub fn StylesSidebar(content: Signal<String>) -> Element {
    rsx! {
        div { class: "styles-sidebar",
            div { class: "sidebar-tabs",
                button { class: "sidebar-tab active", "Styles" }
                button { class: "sidebar-tab", "Properties" }
            }
            input {
                class: "sidebar-search",
                placeholder: "Search styles...",
                r#type: "text",
            }
            div { class: "sidebar-section-label", "Paragraph Styles" }
            div { class: "styles-list",
                for style_def in STYLES {
                    {
                        let prefix = style_def.prefix;
                        rsx! {
                            button {
                                class: "style-item",
                                onclick: move |_| {
                                    if !prefix.is_empty() {
                                        let val = content.read().clone();
                                        let cursor = get_cursor_position();
                                        let line_start = val[..cursor]
                                            .rfind('\n')
                                            .map(|p| p + 1)
                                            .unwrap_or(0);
                                        let mut result = String::with_capacity(val.len() + prefix.len());
                                        result.push_str(&val[..line_start]);
                                        result.push_str(prefix);
                                        result.push_str(&val[line_start..]);
                                        let new_pos = cursor + prefix.len();
                                        content.set(result);
                                        set_cursor_position(new_pos);
                                    }
                                },
                                span { class: "style-item-label", "{style_def.label}" }
                                span { class: "style-item-size", "{style_def.size}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
