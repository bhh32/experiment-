pub(crate) mod api;
pub(crate) mod auth;
pub(crate) mod cache;
pub(crate) mod commands;
pub(crate) mod instance;
pub(crate) mod notifications;
pub(crate) mod util;

use commands::{
    auth::{add_instance, login, logout},
    issues::{create_issue, get_issue, list_issues, post_comment},
    notifications::{list_notifications, mark_read},
    pulls::{get_pull_diff, list_pulls, merge_pull},
    repos::{get_repo, list_repos, search_repos},
    user::{get_profile, list_orgs},
};
use instance::manager::InstanceManager;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub instance_manager: Mutex<InstanceManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let instance_manager = InstanceManager::new()
        .expect("Failed to initialize instance manager");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(AppState {
                instance_manager: Mutex::new(instance_manager),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_instance,
            create_issue,
            get_issue,
            get_profile,
            get_pull_diff,
            get_repo,
            list_issues,
            list_notifications,
            list_orgs,
            list_pulls,
            list_repos,
            login,
            logout,
            mark_read,
            merge_pull,
            post_comment,
            search_repos,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run tauri application");
}
