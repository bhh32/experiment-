use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{self, Issue};
use crate::components::issue_card::IssueCard;
use serde::Serialize;

#[derive(Serialize)]
struct IssueListArgs {
    owner: String,
    repo: String,
    issue_state: Option<String>,
    page: Option<i64>,
}

#[component]
pub fn IssueList() -> impl IntoView {
    let params = use_params_map();

    let issues = LocalResource::new(move || {
        let params = params.get();
        let owner = params.get("owner").unwrap_or_default();
        let name = params.get("name").unwrap_or_default();

        async move {
            let args = IssueListArgs {
                owner: owner.to_string(),
                repo: name.to_string(),
                issue_state: Some("open".to_string()),
                page: Some(1),
            };
            api::tauri_invoke::<_, Vec<Issue>>("list_issues", &args).await
        }
    });

    view! {
        <div class="issue-list-page">
            <h1>"Issues"</h1>

            <Suspense fallback=move || view! { <p>"Loading issues..."</p> }>
                {move || {
                    let params = params.get();
                    let owner = params.get("owner").unwrap_or_default();
                    let name = params.get("name").unwrap_or_default();

                    issues.get().map(|result| match (*result).clone() {
                        Ok(items) => {
                            if items.is_empty() {
                                view! { <p class="empty-state">"No open issues"</p> }.into_any()
                            } else {
                                let views: Vec<_> = items.into_iter().map(|issue| {
                                    view! {
                                        <IssueCard
                                            issue={issue}
                                            owner={owner.to_string()}
                                            repo={name.to_string()}
                                        />
                                    }
                                }).collect();
                                view! { <div class="issue-list">{views}</div> }.into_any()
                            }
                        }
                        Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
