use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{self, Issue};
use serde::Serialize;

#[derive(Serialize)]
struct IssueArgs {
    owner: String,
    repo: String,
    index: i64,
}

#[component]
pub fn IssueDetail() -> impl IntoView {
    let params = use_params_map();

    let issue = LocalResource::new(move || {
        let params = params.get();
        let owner = params.get("owner").unwrap_or_default();
        let name = params.get("name").unwrap_or_default();
        let index: i64 = params.get("index")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        async move {
            let args = IssueArgs {
                owner: owner.to_string(),
                repo: name.to_string(),
                index,
            };
            api::tauri_invoke::<_, Issue>("get_issue", &args).await
        }
    });

    view! {
        <div class="issue-detail-page">
            <Suspense fallback=move || view! { <p>"Loading issue..."</p> }>
                {move || issue.get().map(|result| match (*result).clone() {
                    Ok(issue) => {
                        let state_class = format!("issue-state {}", issue.state);

                        view! {
                            <div class="issue-header">
                                <h1>
                                    {issue.title.clone()}
                                    <span class="issue-number">" #" {issue.number}</span>
                                </h1>
                                <span class={state_class}>{issue.state.clone()}</span>
                            </div>
                            <div class="issue-meta">
                                <span>"Opened by " {issue.user.login.clone()}</span>
                            </div>
                            <div class="issue-body">
                                <p>{issue.body.clone()}</p>
                            </div>
                        }.into_any()
                    }
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
