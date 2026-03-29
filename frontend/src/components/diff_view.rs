use leptos::prelude::*;
use serde::Deserialize;

// These mirror the backend diff structs
#[derive(Debug, Clone, Deserialize)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub status: String,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Hunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiffLine {
    pub kind: String,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[component]
pub fn DiffView(files: Vec<FileDiff>) -> impl IntoView {
    let file_views: Vec<_> = files.into_iter().map(|file| {
        let path = if file.new_path.is_empty() {
            file.old_path.clone()
        } else {
            file.new_path.clone()
        };

        let hunk_views: Vec<_> = file.hunks.into_iter().map(|hunk| {
            let line_views: Vec<_> = hunk.lines.into_iter().map(|line| {
                let class = match line.kind.as_str() {
                    "Addition" => "diff-line addition",
                    "Deletion" => "diff-line deletion",
                    _ => "diff-line context",
                };

                let old_no = line.old_lineno
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                let new_no = line.new_lineno
                    .map(|n| n.to_string())
                    .unwrap_or_default();

                view! {
                    <div class={class}>
                        <span class="line-no old">{old_no}</span>
                        <span class="line-no new">{new_no}</span>
                        <span class="line-content">{line.content}</span>
                    </div>
                }
            }).collect();

            view! {
                <div class="diff-hunk">
                    {line_views}
                </div>
            }
        }).collect();

        view! {
            <details class="diff-file" open>
                <summary class="diff-file-header">{path}</summary>
                <div class="diff-file-content">
                    {hunk_views}
                </div>
            </details>
        }
    }).collect();

    view! {
        <div class="diff-view">
            {file_views}
        </div>
    }
}
