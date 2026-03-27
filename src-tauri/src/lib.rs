pub mod config;
pub mod github;
pub mod google;
pub mod jira;

use config::{
    delete_github_token, delete_jira_token, get_config, get_github_token, get_jira_token,
    has_github_token, has_jira_token, save_config, save_github_token, save_jira_token,
};
use github::{get_pull_requests, test_github_connection};
use google::{
    google_auth_start,
    google_auth_status,
    google_auth_wait,
    google_auth_sign_out,
    google_create_event,
    google_delete_event,
    google_list_calendars,
    google_list_events,
    google_update_event,
    AuthState,
};
use jira::{create_worklog, get_assigned_issues, get_issue_status, search_issues, test_jira_connection};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(AuthState::default())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            get_config,
            save_config,
            save_jira_token,
            get_jira_token,
            delete_jira_token,
            has_jira_token,
            save_github_token,
            get_github_token,
            delete_github_token,
            has_github_token,
            // Jira commands
            get_assigned_issues,
            search_issues,
            create_worklog,
            test_jira_connection,
            get_issue_status,
            // GitHub commands
            get_pull_requests,
            test_github_connection,
            // Google Calendar commands
            google_auth_start,
            google_auth_wait,
            google_auth_status,
            google_auth_sign_out,
            google_list_calendars,
            google_list_events,
            google_create_event,
            google_update_event,
            google_delete_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
