use std::process::Stdio;
use tauri::Emitter;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

use super::error::WorkspaceError;
use super::setup::get_workspace_path;
use super::WorkspaceState;

const PREVIEW_PORT: u16 = 5420;

fn parse_vite_port(line: &str) -> Option<u16> {
    if line.contains("localhost:") {
        if let Some(after) = line.split("localhost:").nth(1) {
            let port_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            return port_str.parse().ok();
        }
    }
    None
}

pub async fn start_vite(
    state: &WorkspaceState,
    app: &tauri::AppHandle,
) -> Result<u16, WorkspaceError> {
    // Check if already running and verify it's responding
    {
        let port = state.vite_port.lock().await;
        if let Some(p) = *port {
            // Verify it's actually responding
            if reqwest::get(&format!("http://localhost:{}", p)).await.is_ok() {
                return Ok(p);
            }
            // Port stored but not responding — clean up
            drop(port);
            let _ = stop_vite(state).await;
        }
    }

    // Kill any leftover vite processes on our port range
    let _ = Command::new("pkill").arg("-f").arg("vite --port 5420").output().await;

    let workspace_path = get_workspace_path()?;

    if !workspace_path.join("package.json").exists() {
        return Err(WorkspaceError::ViteError("Workspace not set up".to_string()));
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "vite",
        "message": "Starting preview server...",
        "percent": 0
    }));

    let mut child = Command::new(&shell)
        .arg("-lc")
        .arg(format!("npx vite --port {} --strictPort false", PREVIEW_PORT))
        .current_dir(&workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|e| WorkspaceError::ViteError(format!("Failed to start Vite: {}", e)))?;

    // Read both stdout and stderr for port detection
    let stdout = child.stdout.take()
        .ok_or_else(|| WorkspaceError::ViteError("Failed to capture stdout".to_string()))?;
    let stderr = child.stderr.take()
        .ok_or_else(|| WorkspaceError::ViteError("Failed to capture stderr".to_string()))?;

    let (port_tx, mut port_rx) = tokio::sync::mpsc::channel::<u16>(1);

    let tx1 = port_tx.clone();
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        let mut sent = false;
        while let Ok(Some(line)) = lines.next_line().await {
            if !sent {
                if let Some(p) = parse_vite_port(&line) {
                    let _ = tx1.send(p).await;
                    sent = true;
                }
            }
            // Keep draining stdout so the pipe stays open and Vite doesn't get SIGPIPE
        }
    });

    let tx2 = port_tx;
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(stderr).lines();
        let mut sent = false;
        while let Ok(Some(line)) = lines.next_line().await {
            if !sent {
                if let Some(p) = parse_vite_port(&line) {
                    let _ = tx2.send(p).await;
                    sent = true;
                }
            }
            // Keep draining stderr so the pipe stays open and Vite doesn't get SIGPIPE
        }
    });

    let port: u16 = match tokio::time::timeout(std::time::Duration::from_secs(30), port_rx.recv()).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            let _ = child.kill().await;
            return Err(WorkspaceError::ViteError("Vite exited without reporting a port".to_string()));
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err(WorkspaceError::ViteError("Timed out waiting for Vite".to_string()));
        }
    };

    // Verify the server is actually responding
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok(resp) = reqwest::get(&format!("http://localhost:{}", port)).await {
            if resp.status().is_success() {
                break;
            }
        }
    }

    // Store state
    {
        let mut vite_proc = state.vite_process.lock().await;
        *vite_proc = Some(child);
    }
    {
        let mut vite_p = state.vite_port.lock().await;
        *vite_p = Some(port);
    }

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "vite",
        "message": format!("Preview running on port {}", port),
        "percent": 100
    }));

    Ok(port)
}

pub async fn stop_vite(state: &WorkspaceState) -> Result<(), WorkspaceError> {
    let mut vite_proc = state.vite_process.lock().await;
    if let Some(mut child) = vite_proc.take() {
        let _ = child.kill().await;
    }

    let mut port = state.vite_port.lock().await;
    *port = None;

    Ok(())
}
