use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::error::ClaudeError;
use super::types::{ChatStreamEvent, ClaudeInfo};

pub async fn check_installed() -> Result<ClaudeInfo, ClaudeError> {
    let which_output = Command::new("which")
        .arg("claude")
        .output()
        .await
        .map_err(|e| ClaudeError::NotInstalled(e.to_string()))?;

    if !which_output.status.success() {
        return Err(ClaudeError::NotInstalled(
            "claude CLI not found in PATH. Install with: npm install -g @anthropic-ai/claude-code"
                .to_string(),
        ));
    }

    let path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();

    let version_output = Command::new("claude")
        .arg("--version")
        .output()
        .await
        .map_err(|e| ClaudeError::NotInstalled(e.to_string()))?;

    let version = String::from_utf8_lossy(&version_output.stdout)
        .trim()
        .to_string();

    Ok(ClaudeInfo { path, version })
}

pub fn spawn_claude(
    repo_path: &Path,
    session_id: &str,
    message: &str,
    is_continuation: bool,
) -> Result<tokio::process::Child, ClaudeError> {
    let mut cmd = Command::new("claude");
    cmd.arg("--print")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--session-id")
        .arg(session_id);

    if is_continuation {
        cmd.arg("--continue");
    }

    cmd.arg(message);

    cmd.current_dir(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| ClaudeError::ProcessError(format!("Failed to spawn claude: {}", e)))?;

    Ok(child)
}

pub async fn stream_output(
    child: &mut tokio::process::Child,
    session_id: &str,
    app: &tauri::AppHandle,
) -> Result<(), ClaudeError> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ClaudeError::ProcessError("Failed to capture stdout".to_string()))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ClaudeError::ProcessError("Failed to capture stderr".to_string()))?;

    let session_id_out = session_id.to_string();
    let app_out = app.clone();

    let session_id_err = session_id.to_string();
    let app_err = app.clone();

    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_out.emit(
                "chat-stream",
                ChatStreamEvent {
                    session_id: session_id_out.clone(),
                    event_type: "content".to_string(),
                    data: line,
                },
            );
        }
    });

    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_err.emit(
                "chat-stream",
                ChatStreamEvent {
                    session_id: session_id_err.clone(),
                    event_type: "error".to_string(),
                    data: line,
                },
            );
        }
    });

    let _ = tokio::join!(stdout_handle, stderr_handle);

    let _ = child.wait().await;

    let _ = app.emit(
        "chat-stream",
        ChatStreamEvent {
            session_id: session_id.to_string(),
            event_type: "done".to_string(),
            data: String::new(),
        },
    );

    Ok(())
}
