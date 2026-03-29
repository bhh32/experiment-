use leptos::prelude::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="loading">
            <div class="loading-spinner"></div>
            <p>"Loading..."</p>
        </div>
    }
}
