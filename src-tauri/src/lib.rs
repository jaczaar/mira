pub mod claude;
pub mod config;
pub mod github;
pub mod google;
pub mod jira;

use claude::{
    cancel_chat_message, check_claude_installed, discard_changes, get_changes_diff,
    send_chat_message, start_chat_session, stop_chat_session, submit_pr, ChatState,
};
use config::{
    delete_github_token, delete_jira_token, get_config, get_github_token, get_jira_token,
    has_github_token, has_jira_token, save_config, save_github_token, save_jira_token,
};
use github::{get_pull_requests, test_github_connection, github_device_flow_start, github_device_flow_poll};
use google::{
    google_auth_sign_out, google_auth_start, google_auth_status, google_auth_wait,
    google_create_event, google_delete_event, google_list_calendars, google_list_events,
    google_update_event, AuthState,
};
use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::Emitter;
use jira::{
    create_worklog, get_assigned_issues, get_issue_status, search_issues, test_jira_connection,
    jira_auth_start, jira_auth_wait, JiraAuthState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_submenu = SubmenuBuilder::new(app, "Mira")
                .about(None)
                .separator()
                .text("settings", "Settings")
                .separator()
                .quit()
                .build()?;

            let menu = MenuBuilder::new(app).item(&app_submenu).build()?;

            app.set_menu(menu)?;

            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                match event.id().as_ref() {
                    "settings" => {
                        let _ = handle.emit("menu-navigate", "settings");
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .manage(AuthState::default())
        .manage(JiraAuthState::default())
        .manage(ChatState::default())
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
            jira_auth_start,
            jira_auth_wait,
            // GitHub commands
            get_pull_requests,
            test_github_connection,
            github_device_flow_start,
            github_device_flow_poll,
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
            // Claude chat commands
            check_claude_installed,
            start_chat_session,
            send_chat_message,
            cancel_chat_message,
            stop_chat_session,
            get_changes_diff,
            submit_pr,
            discard_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
