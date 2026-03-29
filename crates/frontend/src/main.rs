use dioxus::prelude::*;

mod editor;
mod file_menu;
mod toolbar_ui;
mod preview_pane;
mod api;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        editor::EditorView {}
    }
}
