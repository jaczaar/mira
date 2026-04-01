use std::path::PathBuf;
use serde::Serialize;
use tauri::Emitter;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

use super::error::WorkspaceError;
use super::node;

const MIRA_REPO_URL: &str = "https://github.com/jaczaar/mira.git";

pub fn get_mira_dir() -> Result<PathBuf, WorkspaceError> {
    let home = dirs::home_dir().ok_or(WorkspaceError::HomeDirNotFound)?;
    Ok(home.join(".mira"))
}

pub fn get_workspace_path() -> Result<PathBuf, WorkspaceError> {
    Ok(get_mira_dir()?.join("workspace").join("mira"))
}

#[derive(Debug, Serialize)]
pub struct WorkspaceStatus {
    pub claude_available: bool,
    pub node_installed: bool,
    pub node_path: Option<String>,
    pub repo_cloned: bool,
    pub npm_installed: bool,
    pub workspace_path: String,
}

pub async fn check_status() -> Result<WorkspaceStatus, WorkspaceError> {
    let workspace_path = get_workspace_path()?;

    // Check Claude Code CLI — GUI apps don't inherit shell PATH, so fall back to common locations
    let claude_available = crate::claude::process::check_installed().await.is_ok();

    // Check Node.js (local first, then system)
    let (node_installed, node_path) = if let Some(local) = node::get_local_node_binary()? {
        (true, Some(local.to_string_lossy().to_string()))
    } else if let Some(system) = node::check_system_node().await {
        (true, Some(system.to_string_lossy().to_string()))
    } else {
        (false, None)
    };

    // Check if repo is cloned
    let repo_cloned = workspace_path.join(".git").exists();

    // Ensure preview mock is injected
    if repo_cloned {
        let _ = inject_preview_mock(&workspace_path);
    }

    // Check if npm install has been run
    let npm_installed = workspace_path.join("node_modules").exists()
        && workspace_path.join("node_modules").join(".package-lock.json").exists();

    Ok(WorkspaceStatus {
        claude_available,
        node_installed,
        node_path,
        repo_cloned,
        npm_installed,
        workspace_path: workspace_path.to_string_lossy().to_string(),
    })
}

pub async fn clone_repo(app: &tauri::AppHandle) -> Result<(), WorkspaceError> {
    let workspace_path = get_workspace_path()?;

    if workspace_path.join(".git").exists() {
        return Ok(());
    }

    // Create parent directory
    let parent = workspace_path.parent()
        .ok_or_else(|| WorkspaceError::IoError("Invalid workspace path".to_string()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "repo",
        "message": "Cloning Mira repository...",
        "percent": 0
    }));

    let mut child = Command::new("git")
        .arg("clone")
        .arg("--progress")
        .arg("--depth")
        .arg("1")
        .arg(MIRA_REPO_URL)
        .arg(&workspace_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Parse progress from stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        let app_clone = app.clone();

        while let Ok(Some(line)) = lines.next_line().await {
            // Parse git progress: "Receiving objects:  45% (123/274)"
            if let Some(pct_str) = line.split('%').next() {
                if let Some(num_str) = pct_str.rsplit_once(|c: char| !c.is_ascii_digit()) {
                    if let Ok(pct) = num_str.1.parse::<u32>() {
                        let _ = app_clone.emit("workspace-progress", serde_json::json!({
                            "step": "repo",
                            "message": line.trim(),
                            "percent": pct
                        }));
                    }
                }
            }
        }
    }

    let status = child.wait().await
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    if !status.success() {
        // Clean up partial clone
        let _ = std::fs::remove_dir_all(&workspace_path);
        return Err(WorkspaceError::GitError("git clone failed".to_string()));
    }

    // Inject preview mock so the app renders without Tauri IPC
    inject_preview_mock(&workspace_path)?;

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "repo",
        "message": "Repository cloned",
        "percent": 100
    }));

    Ok(())
}

pub async fn run_npm_install(app: &tauri::AppHandle) -> Result<(), WorkspaceError> {
    let workspace_path = get_workspace_path()?;

    if !workspace_path.join("package.json").exists() {
        return Err(WorkspaceError::NpmError("No package.json found in workspace".to_string()));
    }

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "npm",
        "message": "Installing dependencies...",
        "percent": 10
    }));

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    // Use login shell so fnm/nvm paths are available
    let output = Command::new(&shell)
        .arg("-lc")
        .arg("npm install")
        .current_dir(&workspace_path)
        .output()
        .await
        .map_err(|e| WorkspaceError::NpmError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(WorkspaceError::NpmError(format!("npm install failed: {}", stderr)));
    }

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "npm",
        "message": "Dependencies installed",
        "percent": 100
    }));

    Ok(())
}

/// Inject a Tauri API mock so the preview renders without the Tauri IPC bridge.
pub fn inject_preview_mock(workspace: &std::path::Path) -> Result<(), WorkspaceError> {
    let public_dir = workspace.join("public");
    std::fs::create_dir_all(&public_dir)
        .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

    // Write mock script
    let mock_js = r#"// Tauri IPC mock for preview mode
window.__TAURI_INTERNALS__ = {
  invoke: function(cmd, args) {
    // Return sensible defaults so the app renders
    if (cmd === 'get_config') return Promise.resolve({
      jira_url: '', jira_email: '', google_client_id: '', google_client_secret: '',
      selected_calendar: null, sync_frequency: 'manual', auto_sync_on_launch: false,
      jql_filter: null, event_title_template: '', timezone: null, default_event_color: null,
      github_username: '', pr_event_title_template: '', pr_default_event_color: null,
      calendar_colors: {}
    });
    if (cmd === 'google_auth_status') return Promise.resolve([]);
    if (cmd === 'has_jira_token') return Promise.resolve(false);
    if (cmd === 'has_github_token') return Promise.resolve(false);
    if (cmd === 'check_claude_installed') return Promise.reject('preview mode');
    // Default: resolve with null
    return Promise.resolve(null);
  },
  transformCallback: function(cb, once) {
    var id = Math.random().toString(36).slice(2);
    window['_' + id] = cb;
    return Number(id) || 0;
  },
  convertFileSrc: function(path) { return path; },
  unregisterCallback: function() {}
};
"#;

    std::fs::write(public_dir.join("tauri-mock.js"), mock_js)
        .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

    // Inject script tag into index.html if not already present
    let index_path = workspace.join("index.html");
    if index_path.exists() {
        let html = std::fs::read_to_string(&index_path)
            .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

        if !html.contains("tauri-mock.js") {
            let patched = html.replace(
                "<head>",
                "<head>\n    <script src=\"/tauri-mock.js\"></script>"
            );
            std::fs::write(&index_path, patched)
                .map_err(|e| WorkspaceError::IoError(e.to_string()))?;
        }
    }

    Ok(())
}

pub async fn pull_latest() -> Result<(), WorkspaceError> {
    let workspace_path = get_workspace_path()?;

    if !workspace_path.join(".git").exists() {
        return Err(WorkspaceError::GitError("Workspace not cloned".to_string()));
    }

    let output = Command::new("git")
        .arg("pull")
        .arg("--ff-only")
        .current_dir(&workspace_path)
        .output()
        .await
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(WorkspaceError::GitError(format!("git pull failed: {}", stderr)));
    }

    Ok(())
}
