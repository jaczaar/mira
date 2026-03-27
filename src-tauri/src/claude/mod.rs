pub mod error;
pub mod git_ops;
pub mod process;
pub mod session;
pub mod types;

use std::path::PathBuf;

pub use session::ChatState;
use error::ClaudeError;
use types::{ChangeDiff, ClaudeInfo, PRResult};

#[tauri::command]
pub async fn check_claude_installed() -> Result<ClaudeInfo, ClaudeError> {
    process::check_installed().await
}

#[tauri::command]
pub async fn start_chat_session(repo_path: String, state: tauri::State<'_, ChatState>) -> Result<String, ClaudeError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let path = PathBuf::from(&repo_path);

    if !path.exists() {
        return Err(ClaudeError::IoError(format!(
            "Repository path does not exist: {}",
            repo_path
        )));
    }

    let info = session::SessionInfo {
        session_id: session_id.clone(),
        repo_path: path,
        active_process: None,
    };

    state
        .sessions
        .lock()
        .map_err(|e| ClaudeError::ProcessError(e.to_string()))?
        .insert(session_id.clone(), info);

    Ok(session_id)
}

#[tauri::command]
pub async fn send_chat_message(
    session_id: String,
    message: String,
    state: tauri::State<'_, ChatState>,
    app: tauri::AppHandle,
) -> Result<(), ClaudeError> {
    let (repo_path, is_continuation) = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| ClaudeError::SessionNotFound(session_id.clone()))?;
        (session.repo_path.clone(), session.active_process.is_some())
    };

    // Check if this is a continuation (session has had messages before)
    // We track this by checking if there's been a previous message
    let has_previous = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;
        sessions.get(&session_id).is_some()
    };

    let mut child = process::spawn_claude(&repo_path, &session_id, &message, has_previous && is_continuation)?;

    process::stream_output(&mut child, &session_id, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn cancel_chat_message(
    session_id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<(), ClaudeError> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;

    if let Some(session) = sessions.get_mut(&session_id) {
        if let Some(mut child) = session.active_process.take() {
            let _ = child.kill().await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_chat_session(
    session_id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<(), ClaudeError> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;

    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(mut child) = session.active_process.take() {
            let _ = child.kill().await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_changes_diff(
    session_id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<ChangeDiff, ClaudeError> {
    let repo_path = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| ClaudeError::SessionNotFound(session_id.clone()))?;
        session.repo_path.clone()
    };

    git_ops::get_diff(&repo_path).await
}

#[tauri::command]
pub async fn submit_pr(
    session_id: String,
    title: String,
    body: String,
    state: tauri::State<'_, ChatState>,
) -> Result<PRResult, ClaudeError> {
    let repo_path = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| ClaudeError::SessionNotFound(session_id.clone()))?;
        session.repo_path.clone()
    };

    let github_token = crate::config::get_github_token()
        .map_err(|e| ClaudeError::GitHubError(e.to_string()))?
        .ok_or_else(|| {
            ClaudeError::GitHubError(
                "No GitHub token configured. Add one in Settings.".to_string(),
            )
        })?;

    git_ops::create_pr(&repo_path, &title, &body, &github_token).await
}

#[tauri::command]
pub async fn discard_changes(
    session_id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<(), ClaudeError> {
    let repo_path = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|e| ClaudeError::ProcessError(e.to_string()))?;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| ClaudeError::SessionNotFound(session_id.clone()))?;
        session.repo_path.clone()
    };

    git_ops::discard_changes(&repo_path).await
}
