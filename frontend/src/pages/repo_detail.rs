use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::{self, Repository};
use serde::Serialize;

#[derive(Serialize)]
struct RepoArgs {
    owner: String,
    repo: String,
}

#[component]
pub fn RepoDetail() -> impl IntoView {
    let params = use_params_map();

    let repo = LocalResource::new(move || {
        let params = params.get();
        let owner = params.get("owner").unwrap_or_default();
        let name = params.get("name").unwrap_or_default();

        async move {
            let args = RepoArgs {
                owner: owner.to_string(),
                repo: name.to_string(),
            };
            api::tauri_invoke::<_, Repository>("get_repo", &args).await
        }
    });

    view! {
        <div class="repo-detail-page">
            <Suspense fallback=move || view! { <p>"Loading repository..."</p> }>
                {move || repo.get().map(|result| match (*result).clone() {
                    Ok(repo) => {
                        let owner = repo.owner.login.clone();
                        let name = repo.name.clone();

                        view! {
                            <div class="repo-header">
                                <h1>{repo.full_name.clone()}</h1>
                                <p class="repo-description">{repo.description.clone()}</p>
                                <div class="repo-stats">
                                    <span>"⭐ " {repo.stars_count}</span>
                                    <span>"🍴 " {repo.forks_count}</span>
                                    <span>"📋 " {repo.open_issues_count}</span>
                                </div>
                            </div>
                            <div class="repo-actions">
                                <a href={format!("/repos/{owner}/{name}/issues")} class="action-link">
                                    "Issues"
                                </a>
                                <a href={format!("/repos/{owner}/{name}/pulls")} class="action-link">
                                    "Pull Requests"
                                </a>
                            </div>
                        }.into_any()
                    }
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
