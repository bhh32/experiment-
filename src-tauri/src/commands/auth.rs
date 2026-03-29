use crate::api::client::ForgejoClient;
use crate::api::error::ApiError;
use crate::instance::manager::Instance;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    base_url: String,
    token: String,
) -> Result<Instance, ApiError> {
    // Verify the token works by fetching the current user
    let client = ForgejoClient::new(&base_url, &token);
    let user = client.current_user().await?;

    let instance = Instance {
        id: uuid::Uuid::new_v4().to_string(),
        name: base_url.clone(),
        base_url,
        token,
        user_id: Some(user.id),
        username: Some(user.login),
        avatar_url: Some(user.avatar_url),
    };

    {
        let mut manager = state.instance_manager.lock()
            .expect("Failed to lock instance manager");
        manager.add(instance.clone());
        manager.set_active(&instance.id);
    }

    Ok(instance)
}

#[tauri::command]
pub async fn add_instance(
    state: State<'_, AppState>,
    name: String,
    base_url: String,
    token: String,
) -> Result<Instance, ApiError> {
    let client = ForgejoClient::new(&base_url, &token);
    let user = client.current_user().await?;

    let instance = Instance {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        base_url,
        token,
        user_id: Some(user.id),
        username: Some(user.login),
        avatar_url: Some(user.avatar_url),
    };

    {
        let mut manager = state.instance_manager.lock()
            .expect("Failed to lock instance manager");
        manager.add(instance.clone());
    }

    Ok(instance)
}

#[tauri::command]
pub async fn logout(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let mut manager = state.instance_manager.lock()
        .expect("Failed to lock instance manager");
    manager.remove(&instance_id);
    Ok(())
}
