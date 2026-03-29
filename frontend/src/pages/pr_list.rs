use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{self, PullRequest};
use crate::components::pr_card::PrCard;
use serde::Serialize;

#[derive(Serialize)]
struct PullListArgs {
    owner: String,
    repo: String,
    pr_state: Option<String>,
    page: Option<i64>,
}

#[component]
pub fn PrList() -> impl IntoView {
    let params = use_params_map();

    let pulls = LocalResource::new(move || {
        let params = params.get();
        let owner = params.get("owner").unwrap_or_default();
        let name = params.get("name").unwrap_or_default();

        async move {
            let args = PullListArgs {
                owner: owner.to_string(),
                repo: name.to_string(),
                pr_state: Some("open".to_string()),
                page: Some(1),
            };
            api::tauri_invoke::<_, Vec<PullRequest>>("list_pulls", &args).await
        }
    });

    view! {
        <div class="pr-list-page">
            <h1>"Pull Requests"</h1>

            <Suspense fallback=move || view! { <p>"Loading pull requests..."</p> }>
                {move || {
                    let params = params.get();
                    let owner = params.get("owner").unwrap_or_default();
                    let name = params.get("name").unwrap_or_default();

                    pulls.get().map(|result| match (*result).clone() {
                        Ok(items) => {
                            if items.is_empty() {
                                view! { <p class="empty-state">"No open pull requests"</p> }.into_any()
                            } else {
                                let views: Vec<_> = items.into_iter().map(|pr| {
                                    view! {
                                        <PrCard
                                            pr={pr}
                                            owner={owner.to_string()}
                                            repo={name.to_string()}
                                        />
                                    }
                                }).collect();
                                view! { <div class="pr-list">{views}</div> }.into_any()
                            }
                        }
                        Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
