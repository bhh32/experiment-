use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::api::models::{Comment, CreateCommentRequest, CreateIssueRequest, Issue};
use crate::AppState;
use tauri::State;

fn build_client(state: &AppState) -> Result<ForgejoClient, ApiError> {
    let manager = state.instance_manager.lock()
        .expect("Failed to lock instance manager");
    let instance = manager.active_instance()
        .ok_or(ApiError::Unauthorized)?;
    Ok(ForgejoClient::new(&instance.base_url, &instance.token))
}

#[tauri::command]
pub async fn list_issues(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    issue_state: Option<String>,
    page: Option<i64>,
) -> Result<Vec<Issue>, ApiError> {
    let client = build_client(&state)?;
    let state_filter = issue_state.unwrap_or_else(|| "open".to_string());
    client.list_issues(&owner, &repo, &state_filter, page.unwrap_or(1)).await
}

#[tauri::command]
pub async fn get_issue(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    index: i64,
) -> Result<Issue, ApiError> {
    let client = build_client(&state)?;
    client.get_issue(&owner, &repo, index).await
}

#[tauri::command]
pub async fn create_issue(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    title: String,
    body: Option<String>,
) -> Result<Issue, ApiError> {
    let client = build_client(&state)?;
    let request = CreateIssueRequest {
        title,
        body,
        labels: None,
        assignees: None,
    };
    client.create_issue(&owner, &repo, &request).await
}

#[tauri::command]
pub async fn post_comment(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    index: i64,
    body: String,
) -> Result<Comment, ApiError> {
    let client = build_client(&state)?;
    let request = CreateCommentRequest { body };
    client.post_comment(&owner, &repo, index, &request).await
}
