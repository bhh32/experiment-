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
    StyleDef { label: "Heading 4", size: "14pt", prefix: "#### " },
    StyleDef { label: "Body Text", size: "11pt", prefix: "" },
    StyleDef { label: "Caption", size: "10pt", prefix: "" },
    StyleDef { label: "Code", size: "10pt", prefix: "```\n" },
    StyleDef { label: "Quote", size: "11pt", prefix: "> " },
];

#[component]
pub fn StylesSidebar(
    content: Signal<String>,
    font_family: Signal<String>,
    font_size: Signal<f32>,
    line_height: Signal<f32>,
) -> Element {
    let mut active_tab = use_signal(|| "styles".to_string());

    rsx! {
        div { class: "sidebar-tabs",
            button {
                class: if *active_tab.read() == "styles" { "sidebar-tab active" } else { "sidebar-tab" },
                onclick: move |_| active_tab.set("styles".to_string()),
                "Styles"
            }
            button {
                class: if *active_tab.read() == "properties" { "sidebar-tab active" } else { "sidebar-tab" },
                onclick: move |_| active_tab.set("properties".to_string()),
                "Properties"
            }
        }

        if *active_tab.read() == "styles" {
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

        if *active_tab.read() == "properties" {
            div { class: "sidebar-section-label", "Document" }
            div { class: "properties-list",
                div { class: "prop-row",
                    label { class: "prop-label", "Font" }
                    span { class: "prop-value", "{font_family}" }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Size" }
                    span { class: "prop-value", "{font_size}pt" }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Spacing" }
                    span { class: "prop-value",
                        {
                            let lh = *line_height.read();
                            if lh == 1.0 { "Single".to_string() }
                            else if lh == 2.0 { "Double".to_string() }
                            else if lh == 3.0 { "Triple".to_string() }
                            else { format!("{lh}") }
                        }
                    }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Page" }
                    span { class: "prop-value", "8.5\" × 11\" (Letter)" }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Margins" }
                    span { class: "prop-value", "1\" all sides" }
                }
            }

            div { class: "sidebar-section-label", "Statistics" }
            div { class: "properties-list",
                div { class: "prop-row",
                    label { class: "prop-label", "Words" }
                    span { class: "prop-value",
                        {content.read().split_whitespace().count().to_string()}
                    }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Characters" }
                    span { class: "prop-value",
                        {content.read().len().to_string()}
                    }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Lines" }
                    span { class: "prop-value",
                        {content.read().lines().count().to_string()}
                    }
                }
                div { class: "prop-row",
                    label { class: "prop-label", "Paragraphs" }
                    span { class: "prop-value",
                        {
                            content.read()
                                .split("\n\n")
                                .filter(|p| !p.trim().is_empty())
                                .count()
                                .to_string()
                        }
                    }
                }
            }
        }
    }
}
