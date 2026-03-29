use leptos::prelude::*;
use crate::api::Issue;

#[component]
pub fn IssueCard(
    issue: Issue,
    owner: String,
    repo: String,
) -> impl IntoView {
    let detail_url = format!("/repos/{owner}/{repo}/issues/{}", issue.number);
    let label_views: Vec<_> = issue.labels.iter().map(|label| {
        let bg = format!("#{}", label.color);
        view! {
            <span class="label" style:background-color={bg}>{label.name.clone()}</span>
        }
    }).collect();

    view! {
        <a href={detail_url} class="issue-card">
            <div class="issue-header">
                <span class="issue-number">"#" {issue.number}</span>
                <span class="issue-title">{issue.title.clone()}</span>
            </div>
            <div class="issue-meta">
                <span class="issue-author">{issue.user.login.clone()}</span>
                <span class="issue-comments">"💬 " {issue.comments}</span>
            </div>
            <div class="issue-labels">
                {label_views}
            </div>
        </a>
    }
}
