use dioxus::prelude::*;

use crate::file_menu::FileMenu;
use crate::toolbar_ui::ToolbarUi;
use crate::preview_pane::PreviewPane;

#[component]
pub fn EditorView() -> Element {
    let mut content = use_signal(|| String::new());
    let mut file_path = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::new());
    let mut preview_mode = use_signal(|| "markdown".to_string());
    let mut font_family = use_signal(|| "Calibri".to_string());
    let mut line_height = use_signal(|| 1.15f32);

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

        // Tab inserts spaces
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
            FileMenu {
                content: content,
                file_path: file_path,
                status_msg: status_msg,
                font_family: font_family,
                line_height: line_height,
            }
            ToolbarUi {
                content: content,
                font_family: font_family,
                line_height: line_height,
            }
            div { class: "editor-layout",
                div { class: "editor-pane",
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
                div { class: "preview-pane",
                    div { class: "preview-mode-toggle",
                        button {
                            class: if *preview_mode.read() == "markdown" { "mode-btn active" } else { "mode-btn" },
                            onclick: move |_| preview_mode.set("markdown".to_string()),
                            "Markdown"
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
                    PreviewPane {
                        content: content,
                        mode: preview_mode,
                        font_family: font_family,
                        line_height: line_height,
                    }
                }
            }
            if !status_msg.read().is_empty() {
                div { class: "status-bar",
                    "{status_msg}"
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
