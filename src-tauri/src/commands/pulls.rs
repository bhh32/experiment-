use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::api::models::PullRequest;
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
pub async fn list_pulls(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    pr_state: Option<String>,
    page: Option<i64>,
) -> Result<Vec<PullRequest>, ApiError> {
    let client = build_client(&state)?;
    let state_filter = pr_state.unwrap_or_else(|| "open".to_string());
    client.list_pulls(&owner, &repo, &state_filter, page.unwrap_or(1)).await
}

#[tauri::command]
pub async fn get_pull_diff(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    index: i64,
) -> Result<String, ApiError> {
    let client = build_client(&state)?;
    client.get_pull_diff(&owner, &repo, index).await
}

#[tauri::command]
pub async fn merge_pull(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    index: i64,
    method: Option<String>,
) -> Result<(), ApiError> {
    let client = build_client(&state)?;
    let merge_method = method.unwrap_or_else(|| "merge".to_string());
    client.merge_pull(&owner, &repo, index, &merge_method).await
}
