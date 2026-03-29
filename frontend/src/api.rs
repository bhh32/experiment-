use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// Call a Tauri command from the frontend
pub async fn tauri_invoke<A: Serialize, R: DeserializeOwned>(
    command: &str,
    args: &A,
) -> Result<R, String> {
    let args_js = serde_wasm_bindgen::to_value(args)
        .map_err(|e| e.to_string())?;

    let result = invoke(command, args_js)
        .await
        .map_err(|e| {
            // Tauri sends error strings as JsValue
            e.as_string().unwrap_or_else(|| "Unknown error".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| e.to_string())
}

// Shared types that mirror the backend models
// Kept minimal here - only what the frontend needs to deserialize

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub full_name: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub stars_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub updated_at: String,
    pub owner: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub comments: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub merged: bool,
    pub user: User,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub subject: NotificationSubject,
    pub unread: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

// Empty args for commands that take no parameters
#[derive(Serialize)]
pub struct NoArgs {}
