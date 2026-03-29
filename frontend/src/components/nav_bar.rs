use leptos::prelude::*;

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav class="nav-bar">
            <a href="/" class="nav-item">
                <span class="nav-icon">"🏠"</span>
                <span class="nav-label">"Home"</span>
            </a>
            <a href="/repos" class="nav-item">
                <span class="nav-icon">"📦"</span>
                <span class="nav-label">"Repos"</span>
            </a>
            <a href="/notifications" class="nav-item">
                <span class="nav-icon">"🔔"</span>
                <span class="nav-label">"Alerts"</span>
            </a>
            <a href="/profile/me" class="nav-item">
                <span class="nav-icon">"👤"</span>
                <span class="nav-label">"Profile"</span>
            </a>
        </nav>
    }
}
