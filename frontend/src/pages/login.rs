use leptos::prelude::*;
use crate::api;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize)]
struct LoginArgs {
    base_url: String,
    token: String,
}

#[component]
pub fn Login() -> impl IntoView {
    let (base_url, set_base_url) = signal("https://codeberg.org".to_string());
    let (token, set_token) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |_| {
        set_loading.set(true);
        set_error.set(None);

        let url = base_url.get();
        let tok = token.get();

        spawn_local(async move {
            let args = LoginArgs {
                base_url: url,
                token: tok,
            };

            match api::tauri_invoke::<_, api::Instance>("login", &args).await {
                Ok(_instance) => {
                    // Navigate to home on success
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-page">
            <h1>"Connect to Codeberg"</h1>
            <p class="login-subtitle">"Or any Forgejo / Gitea instance"</p>

            <div class="login-form">
                <label for="base-url">"Instance URL"</label>
                <input
                    id="base-url"
                    type="url"
                    prop:value=base_url
                    on:input=move |ev| set_base_url.set(event_target_value(&ev))
                    placeholder="https://codeberg.org"
                />

                <label for="token">"Personal Access Token"</label>
                <input
                    id="token"
                    type="password"
                    prop:value=token
                    on:input=move |ev| set_token.set(event_target_value(&ev))
                    placeholder="Enter your token"
                />

                <button
                    class="login-button"
                    on:click=on_submit
                    disabled=loading
                >
                    {move || if loading.get() { "Connecting..." } else { "Connect" }}
                </button>

                {move || error.get().map(|e| view! {
                    <p class="error-message">{e}</p>
                })}
            </div>
        </div>
    }
}
