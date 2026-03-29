use leptos::prelude::*;
use crate::api::{self, Repository};
use crate::components::repo_card::RepoCard;
use serde::Serialize;

#[derive(Serialize)]
struct ListReposArgs {
    page: Option<i64>,
}

#[component]
pub fn RepoList() -> impl IntoView {
    let repos = LocalResource::new(move || async move {
        let args = ListReposArgs { page: Some(1) };
        api::tauri_invoke::<_, Vec<Repository>>("list_repos", &args).await
    });

    view! {
        <div class="repo-list-page">
            <h1>"Repositories"</h1>

            <Suspense fallback=move || view! { <p>"Loading repositories..."</p> }>
                {move || repos.get().map(|result| match (*result).clone() {
                    Ok(items) => {
                        let views: Vec<_> = items.into_iter().map(|repo| {
                            view! { <RepoCard repo={repo} /> }
                        }).collect();
                        view! { <div class="repo-grid">{views}</div> }.into_any()
                    }
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
