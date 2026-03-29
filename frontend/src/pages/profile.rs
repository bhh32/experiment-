use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{self, User};
use serde::Serialize;

#[derive(Serialize)]
struct ProfileArgs {
    username: Option<String>,
}

#[component]
pub fn Profile() -> impl IntoView {
    let params = use_params_map();

    let user = LocalResource::new(move || {
        let params = params.get();
        let username = params.get("username").map(|s| s.to_string());

        async move {
            // "me" means fetch the authenticated user
            let args = ProfileArgs {
                username: username.filter(|u| u != "me"),
            };
            api::tauri_invoke::<_, User>("get_profile", &args).await
        }
    });

    view! {
        <div class="profile-page">
            <Suspense fallback=move || view! { <p>"Loading profile..."</p> }>
                {move || user.get().map(|result| match (*result).clone() {
                    Ok(user) => view! {
                        <div class="profile-header">
                            <img src={user.avatar_url.clone()} alt="" class="profile-avatar" />
                            <div class="profile-info">
                                <h1>{user.full_name.clone()}</h1>
                                <p class="profile-login">"@" {user.login.clone()}</p>
                            </div>
                        </div>
                    }.into_any(),
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
