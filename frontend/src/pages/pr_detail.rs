use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api;
use serde::Serialize;

#[derive(Serialize)]
struct PullDiffArgs {
    owner: String,
    repo: String,
    index: i64,
}

#[component]
pub fn PrDetail() -> impl IntoView {
    let params = use_params_map();

    let diff = LocalResource::new(move || {
        let params = params.get();
        let owner = params.get("owner").unwrap_or_default();
        let name = params.get("name").unwrap_or_default();
        let index: i64 = params.get("index")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        async move {
            let args = PullDiffArgs {
                owner: owner.to_string(),
                repo: name.to_string(),
                index,
            };
            api::tauri_invoke::<_, String>("get_pull_diff", &args).await
        }
    });

    view! {
        <div class="pr-detail-page">
            <Suspense fallback=move || view! { <p>"Loading pull request..."</p> }>
                {move || diff.get().map(|result| match (*result).clone() {
                    Ok(raw_diff) => {
                        // For now just show the raw diff
                        // Phase 2: parse and render with DiffView component
                        view! {
                            <pre class="raw-diff">{raw_diff}</pre>
                        }.into_any()
                    }
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
