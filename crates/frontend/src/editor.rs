use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::file_menu::FileMenu;
use crate::toolbar_ui::ToolbarUi;
use crate::preview_pane::PreviewPane;
use crate::styles_sidebar::StylesSidebar;
use crate::history::History;

#[component]
pub fn EditorView() -> Element {
    let mut content = use_signal(|| String::new());
    let mut file_path = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::new());
    let mut preview_mode = use_signal(|| "markdown".to_string());
    let mut font_family = use_signal(|| "Times New Roman".to_string());
    let mut font_size = use_signal(|| 12.0f32);
    let mut line_height = use_signal(|| 1.0f32);
    let history = use_signal(|| Rc::new(RefCell::new(History::new())));
    let mut show_find = use_signal(|| false);
    let mut show_replace = use_signal(|| false);
    let mut show_preview = use_signal(|| true);
    let mut show_sidebar = use_signal(|| true);

    // Word and character counts
    let word_count = use_memo(move || {
        let text = content.read();
        text.split_whitespace().count()
    });

    let char_count = use_memo(move || {
        content.read().len()
    });

    // Estimate page count: ~250 words per page (standard double-spaced) + explicit page breaks
    let page_count = use_memo(move || {
        let text = content.read();
        let words = text.split_whitespace().count();
        let explicit_breaks = text.matches("{pagebreak}").count();
        let word_pages = (words as f32 / 250.0).ceil() as usize;
        (word_pages + explicit_breaks).max(1)
    });

    // Auto-save: save to server every 30 seconds if content changed
    let auto_save_content = content;
    let auto_save_path = file_path;
    let mut auto_save_status = status_msg;
    use_future(move || async move {
        loop {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(30_000).await;
            #[cfg(not(target_arch = "wasm32"))]
            std::future::pending::<()>().await;

            let text = auto_save_content.read().clone();
            if text.is_empty() {
                continue;
            }
            let path = auto_save_path.read().clone().unwrap_or_else(|| "untitled.md".to_string());
            match crate::api::save_file(&path, &text).await {
                Ok(()) => {
                    auto_save_status.set(format!("Auto-saved {path}"));
                    // Also save to localStorage as crash recovery
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(storage) = web_sys::window()
                            .and_then(|w| w.local_storage().ok())
                            .flatten()
                        {
                            let _ = storage.set_item("docs_clone_recovery", &text);
                            let _ = storage.set_item("docs_clone_recovery_path", &path);
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });

    // Check for crash recovery on load
    use_future(move || async move {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(recovered)) = storage.get_item("docs_clone_recovery") {
                    if !recovered.is_empty() && content.read().is_empty() {
                        content.set(recovered);
                        if let Ok(Some(path)) = storage.get_item("docs_clone_recovery_path") {
                            file_path.set(Some(path));
                        }
                        status_msg.set("Recovered from auto-save".to_string());
                    }
                }
            }
        }
    });

    let on_content_change = move |evt: Event<FormData>| {
        // Push to undo history before changing
        let old = content.read().clone();
        let cursor = get_cursor_position();
        history.read().borrow_mut().push(&old, cursor);
        content.set(evt.value().clone());
    };

    let on_keydown = move |evt: Event<KeyboardData>| {
        let key = evt.key();
        let ctrl = evt.modifiers().contains(Modifiers::CONTROL)
            || evt.modifiers().contains(Modifiers::META);

        if ctrl {
            match key {
                Key::Character(ref c) if c == "z" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    if let Some((prev_content, prev_cursor)) = history.read().borrow_mut().undo(&val, cursor) {
                        content.set(prev_content);
                        set_cursor_position(prev_cursor);
                    }
                }
                Key::Character(ref c) if c == "y" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    if let Some((next_content, next_cursor)) = history.read().borrow_mut().redo(&val, cursor) {
                        content.set(next_content);
                        set_cursor_position(next_cursor);
                    }
                }
                Key::Character(ref c) if c == "b" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    history.read().borrow_mut().push(&val, cursor);
                    let (new_text, new_pos) = toolbar::formatting::toggle_bold(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "i" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    history.read().borrow_mut().push(&val, cursor);
                    let (new_text, new_pos) = toolbar::formatting::toggle_italic(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "k" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    history.read().borrow_mut().push(&val, cursor);
                    let (new_text, new_pos) = toolbar::formatting::insert_link(&val, cursor);
                    content.set(new_text);
                    set_cursor_position(new_pos);
                }
                Key::Character(ref c) if c == "f" => {
                    evt.prevent_default();
                    show_find.set(true);
                }
                Key::Character(ref c) if c == "h" => {
                    evt.prevent_default();
                    show_find.set(true);
                    show_replace.set(true);
                }
                Key::Character(ref c) if c == "p" => {
                    evt.prevent_default();
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(window) = web_sys::window() {
                            let _ = window.print();
                        }
                    }
                }
                Key::Character(ref c) if c == "u" => {
                    evt.prevent_default();
                    let val = content.read().clone();
                    let cursor = get_cursor_position();
                    history.read().borrow_mut().push(&val, cursor);
                    let (start, end) = get_selection_range();
                    if start < end && end <= val.len() {
                        let mut result = String::with_capacity(val.len() + 4);
                        result.push_str(&val[..start]);
                        result.push_str("__");
                        result.push_str(&val[start..end]);
                        result.push_str("__");
                        result.push_str(&val[end..]);
                        content.set(result);
                        set_cursor_position(end + 4);
                    } else {
                        let cursor = start;
                        let mut result = String::with_capacity(val.len() + 4);
                        result.push_str(&val[..cursor]);
                        result.push_str("____");
                        result.push_str(&val[cursor..]);
                        content.set(result);
                        set_cursor_position(cursor + 2);
                    }
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

            // Smart tab: if current line starts with a list marker, indent it
            let line_start = val[..cursor].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line = &val[line_start..val[line_start..].find('\n').map(|p| line_start + p).unwrap_or(val.len())];
            let trimmed = line.trim_start();
            let is_list = trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("1. ")
                || trimmed.starts_with("[ ] ")
                || trimmed.starts_with("[x] ");

            if is_list {
                // Indent the entire line by 4 spaces
                history.read().borrow_mut().push(&val, cursor);
                let mut new_text = String::with_capacity(val.len() + 4);
                new_text.push_str(&val[..line_start]);
                new_text.push_str("    ");
                new_text.push_str(&val[line_start..]);
                content.set(new_text);
                set_cursor_position(cursor + 4);
            } else {
                // Regular tab: insert 4 spaces at cursor
                history.read().borrow_mut().push(&val, cursor);
                let mut new_text = String::with_capacity(val.len() + 4);
                new_text.push_str(&val[..cursor]);
                new_text.push_str("    ");
                new_text.push_str(&val[cursor..]);
                content.set(new_text);
                set_cursor_position(cursor + 4);
            }
        }
    };

    // Set up drag-and-drop on document body via web_sys
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            let dragover = Closure::wrap(Box::new(|e: web_sys::DragEvent| {
                e.prevent_default();
            }) as Box<dyn Fn(_)>);

            let drop_handler = Closure::wrap(Box::new(move |e: web_sys::DragEvent| {
                e.prevent_default();
                if let Some(dt) = e.data_transfer() {
                    if let Some(files) = dt.files() {
                        if files.length() > 0 {
                            if let Some(file) = files.get(0) {
                                let name = file.name();
                                // Use gloo_file to read the file via JS Promise
                                let blob: web_sys::Blob = file.into();
                                let fname = name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let promise = if fname.ends_with(".docx") {
                                        blob.array_buffer()
                                    } else {
                                        blob.text()
                                    };
                                    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
                                    if let Ok(val) = result {
                                        if fname.ends_with(".md") || fname.ends_with(".txt") || fname.ends_with(".markdown") {
                                            if let Some(text) = val.as_string() {
                                                content.set(text);
                                                file_path.set(Some(fname.clone()));
                                                status_msg.set(format!("Opened {fname}"));
                                            }
                                        } else if fname.ends_with(".docx") {
                                            let arr = js_sys::Uint8Array::new(&val);
                                            let upload_blob = web_sys::Blob::new_with_u8_array_sequence(
                                                &js_sys::Array::of1(&arr.into()),
                                            ).unwrap();
                                            let fname2 = fname.clone();
                                            let _ = gloo_net::http::Request::put(&format!("/api/files/{fname2}"))
                                                .body(upload_blob)
                                                .unwrap()
                                                .send()
                                                .await;
                                            match crate::api::read_file(&fname2).await {
                                                Ok(md) => {
                                                    content.set(md);
                                                    file_path.set(Some(fname2.clone()));
                                                    status_msg.set(format!("Opened {fname2}"));
                                                }
                                                Err(e) => status_msg.set(format!("Failed: {e}")),
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }) as Box<dyn Fn(_)>);

            if let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) {
                let _ = body.add_event_listener_with_callback("dragover", dragover.as_ref().unchecked_ref());
                let _ = body.add_event_listener_with_callback("drop", drop_handler.as_ref().unchecked_ref());
            }

            dragover.forget();
            drop_handler.forget();
        });
    }

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
                show_find: show_find,
                show_replace: show_replace,
            }

            // Toolbar (font, size, formatting, alignment)
            ToolbarUi {
                content: content,
                font_family: font_family,
                font_size: font_size,
                line_height: line_height,
            }

            // Find/Replace bar
            crate::find_replace::FindReplace {
                content: content,
                show_find: show_find,
                show_replace: show_replace,
            }

            // Main layout: editor | preview | styles sidebar
            div { class: "main-layout",
                div { class: "editor-preview-area",
                    // Markdown source editor
                    div {
                        class: if *show_preview.read() { "editor-pane" } else { "editor-pane editor-pane-full" },
                        div { class: "editor-pane-header",
                            span { "Source" }
                        }
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

                    // Preview toggle on the border
                    button {
                        class: "border-toggle preview-border-toggle",
                        title: if *show_preview.read() { "Hide Preview" } else { "Show Preview" },
                        onclick: move |_| {
                            let current = *show_preview.read();
                            show_preview.set(!current);
                        },
                        if *show_preview.read() { "›" } else { "‹" }
                    }

                    // Document preview (collapsible)
                    if *show_preview.read() {
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
                }

                // Sidebar toggle on the border
                button {
                    class: "border-toggle sidebar-border-toggle",
                    title: if *show_sidebar.read() { "Hide Sidebar" } else { "Show Sidebar" },
                    onclick: move |_| {
                        let current = *show_sidebar.read();
                        show_sidebar.set(!current);
                    },
                    if *show_sidebar.read() { "›" } else { "‹" }
                }

                // Styles/Properties sidebar (collapsible)
                if *show_sidebar.read() {
                    div { class: "styles-sidebar",
                        StylesSidebar {
                            content: content,
                            font_family: font_family,
                            font_size: font_size,
                            line_height: line_height,
                        }
                    }
                }
            }

            // Status bar
            div { class: "status-bar",
                div { class: "status-left",
                    span { "Page {page_count}" }
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

/// Get the selection range (start, end). If no selection, start == end == cursor.
pub fn get_selection_range() -> (usize, usize) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        if let Some(el) = doc.get_element_by_id("editor-textarea") {
            if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                let start = textarea.selection_start().unwrap_or(Some(0)).unwrap_or(0) as usize;
                let end = textarea.selection_end().unwrap_or(Some(0)).unwrap_or(0) as usize;
                return (start, end);
            }
        }
        (0, 0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        (0, 0)
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
