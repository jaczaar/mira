pub mod error;
pub mod node;
pub mod setup;
pub mod vite;

use tokio::sync::Mutex;

use error::WorkspaceError;
use setup::WorkspaceStatus;

pub struct WorkspaceState {
    pub vite_process: Mutex<Option<tokio::process::Child>>,
    pub vite_port: Mutex<Option<u16>>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            vite_process: Mutex::new(None),
            vite_port: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn workspace_check_status() -> Result<WorkspaceStatus, WorkspaceError> {
    setup::check_status().await
}

#[tauri::command]
pub async fn workspace_setup_node(app: tauri::AppHandle) -> Result<(), WorkspaceError> {
    node::download_node(&app).await?;
    Ok(())
}

#[tauri::command]
pub async fn workspace_clone_repo(app: tauri::AppHandle) -> Result<(), WorkspaceError> {
    setup::clone_repo(&app).await
}

#[tauri::command]
pub async fn workspace_npm_install(app: tauri::AppHandle) -> Result<(), WorkspaceError> {
    setup::run_npm_install(&app).await
}

#[tauri::command]
pub async fn workspace_start_vite(
    state: tauri::State<'_, WorkspaceState>,
    app: tauri::AppHandle,
) -> Result<u16, WorkspaceError> {
    vite::start_vite(&state, &app).await
}

#[tauri::command]
pub async fn workspace_stop_vite(
    state: tauri::State<'_, WorkspaceState>,
) -> Result<(), WorkspaceError> {
    vite::stop_vite(&state).await
}

#[tauri::command]
pub async fn workspace_get_path() -> Result<String, WorkspaceError> {
    let path = setup::get_workspace_path()?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn workspace_pull_latest() -> Result<(), WorkspaceError> {
    setup::pull_latest().await
}

