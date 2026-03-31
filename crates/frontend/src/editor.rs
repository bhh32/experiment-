use dioxus::prelude::*;

use crate::file_menu::FileMenu;
use crate::toolbar_ui::ToolbarUi;
use crate::preview_pane::PreviewPane;
use crate::styles_sidebar::StylesSidebar;

#[component]
pub fn EditorView() -> Element {
    let mut content = use_signal(|| String::new());
    let mut file_path = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::new());
    let mut preview_mode = use_signal(|| "markdown".to_string());
    let mut font_family = use_signal(|| "Times New Roman".to_string());
    let mut font_size = use_signal(|| 12.0f32);
    let mut line_height = use_signal(|| 1.0f32);

    // Word and character counts
    let word_count = use_memo(move || {
        let text = content.read();
        text.split_whitespace().count()
    });

    let char_count = use_memo(move || {
        content.read().len()
    });

    let on_content_change = move |evt: Event<FormData>| {
        content.set(evt.value().clone());
    };

    let on_keydown = move |evt: Event<KeyboardData>| {
        let key = evt.key();
        let ctrl = evt.modifiers().contains(Modifiers::CONTROL)
            || evt.modifiers().contains(Modifiers::META);

        if ctrl {
            match key {
                Key::Character(ref c) if c == "b" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_bold(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "i" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::toggle_italic(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "k" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    let (new_text, new_pos) = toolbar::formatting::insert_link(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "s" => {
                    evt.prevent_default();
                    let path = file_path.read().clone();
                    let text = content.read().clone();
                    spawn(async move {
                        let save_path = path.unwrap_or_else(|| "untitled.md".to_string());
                        match crate::api::save_file(&save_path, &text).await {
                            Ok(()) => status_msg.set(format!("Saved {save_path}")),
                            Err(e) => status_msg.set(format!("Save failed: {e}")),
                        }
                    });
                }
                _ => {}
            }
        }

        if key == Key::Tab {
            evt.prevent_default();
            let val = content.read().clone();
            let cursor = get_cursor_position();
            let mut new_text = String::with_capacity(val.len() + 4);
            new_text.push_str(&val[..cursor]);
            new_text.push_str("    ");
            new_text.push_str(&val[cursor..]);
            content.set(new_text);
            set_cursor_position(cursor + 4);
        }
    };

    rsx! {
        div { class: "app-container",
            // Menu bar (File, Edit, View, etc.)
            FileMenu {
                content: content,
                file_path: file_path,
                status_msg: status_msg,
                font_family: font_family,
                font_size: font_size,
                line_height: line_height,
            }

            // Toolbar (font, size, formatting, alignment)
            ToolbarUi {
                content: content,
                font_family: font_family,
                font_size: font_size,
                line_height: line_height,
            }

            // Main layout: editor | preview | styles sidebar
            div { class: "main-layout",
                div { class: "editor-preview-area",
                    // Markdown source editor
                    div { class: "editor-pane",
                        div { class: "editor-pane-header", "Source" }
                        textarea {
                            id: "editor-textarea",
                            class: "editor-textarea",
                            spellcheck: "true",
                            placeholder: "Start writing markdown...",
                            value: "{content}",
                            oninput: on_content_change,
                            onkeydown: on_keydown,
                        }
                    }

                    // Document preview
                    div { class: "preview-pane",
                        div { class: "preview-pane-header",
                            span { class: "preview-pane-label", "Preview" }
                            div { class: "preview-mode-toggle",
                                button {
                                    class: if *preview_mode.read() == "markdown" { "mode-btn active" } else { "mode-btn" },
                                    onclick: move |_| preview_mode.set("markdown".to_string()),
                                    "Print"
                                }
                                button {
                                    class: if *preview_mode.read() == "docx" { "mode-btn active" } else { "mode-btn" },
                                    onclick: move |_| preview_mode.set("docx".to_string()),
                                    "DOCX"
                                }
                                button {
                                    class: if *preview_mode.read() == "odt" { "mode-btn active" } else { "mode-btn" },
                                    onclick: move |_| preview_mode.set("odt".to_string()),
                                    "ODF"
                                }
                            }
                        }
                        div { class: "preview-scroll-area",
                            PreviewPane {
                                content: content,
                                mode: preview_mode,
                                font_family: font_family,
                                font_size: font_size,
                                line_height: line_height,
                            }
                        }
                    }
                }

                // Styles sidebar
                StylesSidebar {
                    content: content,
                }
            }

            // Status bar
            div { class: "status-bar",
                div { class: "status-left",
                    span { "Words: {word_count}" }
                    span { "Characters: {char_count}" }
                    if !status_msg.read().is_empty() {
                        span { class: "status-msg", "{status_msg}" }
                    }
                }
                div { class: "status-right",
                    span { "English (US)" }
                }
            }
        }
    }
}

pub fn get_cursor_position() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        if let Some(el) = doc.get_element_by_id("editor-textarea") {
            if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                return textarea.selection_start().unwrap_or(Some(0)).unwrap_or(0) as usize;
            }
        }
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn set_cursor_position(pos: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        if let Some(el) = doc.get_element_by_id("editor-textarea") {
            if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                let _ = textarea.set_selection_start(Some(pos as u32));
                let _ = textarea.set_selection_end(Some(pos as u32));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = pos;
    }
}
