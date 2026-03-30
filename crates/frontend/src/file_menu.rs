use dioxus::prelude::*;

use shared::FileEntry;

#[component]
pub fn FileMenu(
    content: Signal<String>,
    file_path: Signal<Option<String>>,
    status_msg: Signal<String>,
    font_family: Signal<String>,
    font_size: Signal<f32>,
    line_height: Signal<f32>,
) -> Element {
    let mut show_open_dialog = use_signal(|| false);
    let mut files_list = use_signal(Vec::<FileEntry>::new);
    let mut open_menu = use_signal(|| Option::<String>::None);

    let close_menu = move |_: Event<MouseData>| {
        open_menu.set(None);
    };

    rsx! {
        div { class: "menu-bar",
            div { class: "menu-bar-left",
                // File menu
                div { class: "dropdown-container",
                    button {
                        class: "menu-item",
                        onclick: move |_| {
                            let current = open_menu.read().clone();
                            if current.as_deref() == Some("file") {
                                open_menu.set(None);
                            } else {
                                open_menu.set(Some("file".to_string()));
                            }
                        },
                        "File"
                    }
                    if open_menu.read().as_deref() == Some("file") {
                        div { class: "dropdown-menu",
                            button {
                                class: "dropdown-item",
                                onclick: move |_| {
                                    open_menu.set(None);
                                    spawn(async move {
                                        match crate::api::list_files().await {
                                            Ok(files) => {
                                                files_list.set(files);
                                                show_open_dialog.set(true);
                                            }
                                            Err(e) => status_msg.set(format!("Failed to list files: {e}")),
                                        }
                                    });
                                },
                                "Open"
                                span { class: "dropdown-item-shortcut", "Ctrl+O" }
                            }
                            button {
                                class: "dropdown-item",
                                id: "save-btn",
                                onclick: move |_| {
                                    open_menu.set(None);
                                    let path = file_path.read().clone();
                                    let text = content.read().clone();
                                    spawn(async move {
                                        let save_path = path.unwrap_or_else(|| "untitled.md".to_string());
                                        match crate::api::save_file(&save_path, &text).await {
                                            Ok(()) => {
                                                file_path.set(Some(save_path.clone()));
                                                status_msg.set(format!("Saved {save_path}"));
                                            }
                                            Err(e) => status_msg.set(format!("Save failed: {e}")),
                                        }
                                    });
                                },
                                "Save"
                                span { class: "dropdown-item-shortcut", "Ctrl+S" }
                            }
                            button {
                                class: "dropdown-item",
                                id: "save-as-btn",
                                onclick: move |_| {
                                    open_menu.set(None);
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let text = content.read().clone();
                                        spawn(async move {
                                            let window = web_sys::window().unwrap();
                                            if let Some(name) = window.prompt_with_message("Save as:").ok().flatten() {
                                                if !name.is_empty() {
                                                    let name = if !name.ends_with(".md") {
                                                        format!("{name}.md")
                                                    } else {
                                                        name
                                                    };
                                                    match crate::api::save_file(&name, &text).await {
                                                        Ok(()) => {
                                                            file_path.set(Some(name.clone()));
                                                            status_msg.set(format!("Saved as {name}"));
                                                        }
                                                        Err(e) => status_msg.set(format!("Save As failed: {e}")),
                                                    }
                                                }
                                            }
                                        });
                                    }
                                },
                                "Save As..."
                            }
                            div { class: "dropdown-divider" }
                            button {
                                class: "dropdown-item",
                                id: "export-docx-btn",
                                onclick: move |_| {
                                    open_menu.set(None);
                                    let text = content.read().clone();
                                    let f = font_family.read().clone();
                                    let fs = *font_size.read();
                                    let lh = *line_height.read();
                                    let fname = file_path
                                        .read()
                                        .as_ref()
                                        .map(|p| p.replace(".md", ".docx"))
                                        .unwrap_or_else(|| "document.docx".to_string());
                                    spawn(async move {
                                        match crate::api::convert(&text, "markdown", "docx", Some(&f), Some(fs), Some(lh)).await {
                                            Ok(resp) => {
                                                download_base64(&resp.content, &fname, "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
                                                status_msg.set(format!("Exported {fname}"));
                                            }
                                            Err(e) => status_msg.set(format!("Export DOCX failed: {e}")),
                                        }
                                    });
                                },
                                "Export as DOCX"
                            }
                            button {
                                class: "dropdown-item",
                                id: "export-odf-btn",
                                onclick: move |_| {
                                    open_menu.set(None);
                                    let text = content.read().clone();
                                    let fname = file_path
                                        .read()
                                        .as_ref()
                                        .map(|p| p.replace(".md", ".odt"))
                                        .unwrap_or_else(|| "document.odt".to_string());
                                    spawn(async move {
                                        match crate::api::convert(&text, "markdown", "odt", None, None, None).await {
                                            Ok(resp) => {
                                                download_base64(&resp.content, &fname, "application/vnd.oasis.opendocument.text");
                                                status_msg.set(format!("Exported {fname}"));
                                            }
                                            Err(e) => status_msg.set(format!("Export ODF failed: {e}")),
                                        }
                                    });
                                },
                                "Export as ODF"
                            }
                        }
                    }
                }

                // Edit menu
                button { class: "menu-item", onclick: close_menu, "Edit" }
                button { class: "menu-item", onclick: close_menu, "View" }
                button { class: "menu-item", onclick: close_menu, "Insert" }
                button { class: "menu-item", onclick: close_menu, "Format" }
                button { class: "menu-item", onclick: close_menu, "Styles" }
                button { class: "menu-item", onclick: close_menu, "Table" }
                button { class: "menu-item", onclick: close_menu, "Tools" }
                button { class: "menu-item", onclick: close_menu, "Help" }
            }

            div { class: "menu-bar-right",
                span { class: "file-name-display",
                    {
                        let name = file_path.read().clone().unwrap_or_else(|| "untitled.md".to_string());
                        name
                    }
                }
            }
        }

        // Open File dialog
        if *show_open_dialog.read() {
            div { class: "dialog-overlay", onclick: move |_| show_open_dialog.set(false),
                div { class: "dialog", onclick: move |evt| evt.stop_propagation(),
                    h3 { "Open File" }
                    div { class: "file-list",
                        for file in files_list.read().iter() {
                            {
                                let name = file.name.clone();
                                let path = file.path.clone();
                                rsx! {
                                    button {
                                        class: "file-list-item",
                                        onclick: move |_| {
                                            let p = path.clone();
                                            spawn(async move {
                                                match crate::api::read_file(&p).await {
                                                    Ok(text) => {
                                                        content.set(text);
                                                        file_path.set(Some(p.clone()));
                                                        show_open_dialog.set(false);
                                                        status_msg.set(format!("Opened {p}"));
                                                    }
                                                    Err(e) => status_msg.set(format!("Open failed: {e}")),
                                                }
                                            });
                                        },
                                        "{name}"
                                    }
                                }
                            }
                        }
                        if files_list.read().is_empty() {
                            p { class: "empty-msg", "No documents found" }
                        }
                    }
                    button { class: "dialog-close", onclick: move |_| show_open_dialog.set(false), "Cancel" }
                }
            }
        }
    }
}

fn download_base64(_data: &str, _filename: &str, _mime: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::{Array, Uint8Array};
        use wasm_bindgen::JsCast;
        use web_sys::{Blob, BlobPropertyBag, Url};

        let decoded = base64_decode(_data);
        let uint8arr = Uint8Array::new_with_length(decoded.len() as u32);
        uint8arr.copy_from(&decoded);

        let array = Array::new();
        array.push(&uint8arr.buffer());

        let mut opts = BlobPropertyBag::new();
        opts.type_(_mime);
        let blob = Blob::new_with_buffer_source_sequence_and_options(&array, &opts).unwrap();
        let url = Url::create_object_url_with_blob(&blob).unwrap();

        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        let a = doc
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .unwrap();
        a.set_href(&url);
        a.set_download(_filename);
        a.click();
        let _ = Url::revoke_object_url(&url);
    }
}

#[cfg(target_arch = "wasm32")]
fn base64_decode(input: &str) -> Vec<u8> {
    const TABLE: &[u8; 128] = &{
        let mut t = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            t[chars[i] as usize] = i as u8;
            i += 1;
        }
        t
    };

    let mut output = Vec::new();
    let bytes: Vec<u8> = input.bytes().filter(|&b| b != b'=' && b != b'\n' && b != b'\r').collect();

    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 { break; }
        let b0 = TABLE[chunk[0] as usize] as u32;
        let b1 = TABLE[chunk[1] as usize] as u32;
        output.push(((b0 << 2) | (b1 >> 4)) as u8);

        if chunk.len() > 2 {
            let b2 = TABLE[chunk[2] as usize] as u32;
            output.push((((b1 & 0xF) << 4) | (b2 >> 2)) as u8);

            if chunk.len() > 3 {
                let b3 = TABLE[chunk[3] as usize] as u32;
                output.push((((b2 & 0x3) << 6) | b3) as u8);
            }
        }
    }
    output
}
