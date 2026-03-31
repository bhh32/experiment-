use dioxus::prelude::*;

use crate::editor::{get_cursor_position, set_cursor_position};

#[component]
pub fn FindReplace(
    content: Signal<String>,
    show_find: Signal<bool>,
    show_replace: Signal<bool>,
) -> Element {
    let mut find_text = use_signal(|| String::new());
    let mut replace_text = use_signal(|| String::new());
    let mut match_count = use_signal(|| 0usize);
    let mut current_match = use_signal(|| 0usize);

    // Count matches whenever find text or content changes
    let count = use_memo(move || {
        let needle = find_text.read().clone();
        let haystack = content.read().clone();
        if needle.is_empty() {
            return 0;
        }
        haystack.matches(&*needle).count()
    });

    let mut do_find_next = move || {
        let needle = find_text.read().clone();
        let haystack = content.read().clone();
        if needle.is_empty() {
            return;
        }
        let cursor = get_cursor_position();
        // Find next occurrence after cursor
        if let Some(pos) = haystack[cursor..].find(&*needle) {
            let abs_pos = cursor + pos;
            set_cursor_position(abs_pos);
            // Select the match
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("editor-textarea"))
                {
                    if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        let _ = textarea.set_selection_start(Some(abs_pos as u32));
                        let _ = textarea.set_selection_end(Some((abs_pos + needle.len()) as u32));
                        let _ = textarea.focus();
                    }
                }
            }
            match_count.set(*count.read());
        } else if let Some(pos) = haystack.find(&*needle) {
            // Wrap around
            set_cursor_position(pos);
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("editor-textarea"))
                {
                    if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        let _ = textarea.set_selection_start(Some(pos as u32));
                        let _ = textarea.set_selection_end(Some((pos + needle.len()) as u32));
                        let _ = textarea.focus();
                    }
                }
            }
            match_count.set(*count.read());
        }
    };

    let replace_current = move |_| {
        let needle = find_text.read().clone();
        let replacement = replace_text.read().clone();
        let haystack = content.read().clone();
        if needle.is_empty() {
            return;
        }
        let cursor = get_cursor_position();
        // Check if the text at cursor matches
        if haystack[cursor..].starts_with(&*needle) {
            let mut result = String::with_capacity(haystack.len());
            result.push_str(&haystack[..cursor]);
            result.push_str(&replacement);
            result.push_str(&haystack[cursor + needle.len()..]);
            content.set(result);
            set_cursor_position(cursor + replacement.len());
            match_count.set(content.read().matches(&*needle).count());
        }
    };

    let replace_all = move |_| {
        let needle = find_text.read().clone();
        let replacement = replace_text.read().clone();
        if needle.is_empty() {
            return;
        }
        let haystack = content.read().clone();
        let new_content = haystack.replace(&*needle, &replacement);
        content.set(new_content);
        match_count.set(0);
    };

    let close = move |_| {
        show_find.set(false);
        show_replace.set(false);
    };

    if !*show_find.read() {
        return rsx! {};
    }

    rsx! {
        div { class: "find-bar",
            div { class: "find-row",
                input {
                    class: "find-input",
                    placeholder: "Find...",
                    value: "{find_text}",
                    oninput: move |evt| {
                        find_text.set(evt.value().clone());
                        match_count.set(*count.read());
                    },
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter {
                            do_find_next();
                        }
                        if evt.key() == Key::Escape {
                            show_find.set(false);
                            show_replace.set(false);
                        }
                    },
                }
                span { class: "find-count", "{count} matches" }
                button { class: "find-btn", onclick: move |_| do_find_next(), "Next" }
                button { class: "find-btn", onclick: close, "\u{2715}" }
            }
            if *show_replace.read() {
                div { class: "find-row",
                    input {
                        class: "find-input",
                        placeholder: "Replace with...",
                        value: "{replace_text}",
                        oninput: move |evt| replace_text.set(evt.value().clone()),
                    }
                    button { class: "find-btn", onclick: replace_current, "Replace" }
                    button { class: "find-btn", onclick: replace_all, "All" }
                }
            }
        }
    }
}
