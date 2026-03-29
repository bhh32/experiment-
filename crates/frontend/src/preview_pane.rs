use dioxus::prelude::*;

#[component]
pub fn PreviewPane(content: Signal<String>, mode: Signal<String>) -> Element {
    let html = use_memo(move || {
        let text = content.read().clone();
        let m = mode.read().clone();
        match m.as_str() {
            "docx" => markdown_preview::render_docx_preview(&text),
            "odt" => markdown_preview::render_odt_preview(&text),
            _ => markdown_preview::render_preview(&text),
        }
    });

    rsx! {
        div {
            class: "preview-content",
            dangerous_inner_html: "{html}",
        }
    }
}
