use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::api::models::Repository;
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
pub async fn list_repos(
    state: State<'_, AppState>,
    page: Option<i64>,
) -> Result<Vec<Repository>, ApiError> {
    let client = build_client(&state)?;
    client.list_repos(page.unwrap_or(1), 20).await
}

#[tauri::command]
pub async fn get_repo(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
) -> Result<Repository, ApiError> {
    let client = build_client(&state)?;
    client.get_repo(&owner, &repo).await
}

#[tauri::command]
pub async fn search_repos(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Repository>, ApiError> {
    let client = build_client(&state)?;
    client.search_repos(&query).await
}
