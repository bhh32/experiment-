use leptos::prelude::*;
use crate::api::Repository;

#[component]
pub fn RepoCard(repo: Repository) -> impl IntoView {
    let detail_url = format!("/repos/{}", repo.full_name);

    view! {
        <a href={detail_url} class="repo-card">
            <div class="repo-card-header">
                <img src={repo.owner.avatar_url.clone()} alt="" class="repo-avatar" />
                <span class="repo-name">{repo.full_name.clone()}</span>
            </div>
            <p class="repo-description">{repo.description.clone()}</p>
            <div class="repo-stats">
                <span class="stat">"⭐ " {repo.stars_count}</span>
                <span class="stat">"🍴 " {repo.forks_count}</span>
                <span class="stat">"📋 " {repo.open_issues_count}</span>
            </div>
        </a>
    }
}
