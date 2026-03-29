use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::api::models::Notification;
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
pub async fn list_notifications(
    state: State<'_, AppState>,
) -> Result<Vec<Notification>, ApiError> {
    let client = build_client(&state)?;
    client.list_notifications(None).await
}

#[tauri::command]
pub async fn mark_read(
    state: State<'_, AppState>,
) -> Result<(), ApiError> {
    let client = build_client(&state)?;
    client.mark_notifications_read().await
}
