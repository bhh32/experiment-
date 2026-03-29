use leptos::prelude::*;
use crate::api::PullRequest;

#[component]
pub fn PrCard(
    pr: PullRequest,
    owner: String,
    repo: String,
) -> impl IntoView {
    let detail_url = format!("/repos/{owner}/{repo}/pulls/{}", pr.number);

    let status_class = if pr.merged {
        "pr-status merged"
    } else if pr.state == "closed" {
        "pr-status closed"
    } else {
        "pr-status open"
    };

    let status_text = if pr.merged {
        "Merged"
    } else if pr.state == "closed" {
        "Closed"
    } else {
        "Open"
    };

    view! {
        <a href={detail_url} class="pr-card">
            <div class="pr-header">
                <span class={status_class}>{status_text}</span>
                <span class="pr-number">"#" {pr.number}</span>
                <span class="pr-title">{pr.title.clone()}</span>
            </div>
            <div class="pr-meta">
                <span class="pr-author">{pr.user.login.clone()}</span>
            </div>
        </a>
    }
}
