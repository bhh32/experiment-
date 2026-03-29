use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::api::models::{Organization, User};
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
pub async fn get_profile(
    state: State<'_, AppState>,
    username: Option<String>,
) -> Result<User, ApiError> {
    let client = build_client(&state)?;
    match username {
        Some(name) => client.get_user(&name).await,
        None => client.current_user().await,
    }
}

#[tauri::command]
pub async fn list_orgs(
    state: State<'_, AppState>,
) -> Result<Vec<Organization>, ApiError> {
    let client = build_client(&state)?;
    client.list_user_orgs().await
}
