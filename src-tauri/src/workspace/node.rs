use std::path::PathBuf;
use tokio::process::Command;
use tauri::Emitter;

use super::error::WorkspaceError;
use super::setup::get_mira_dir;

const NODE_VERSION: &str = "22.15.0";

pub fn get_node_dir() -> Result<PathBuf, WorkspaceError> {
    Ok(get_mira_dir()?.join("tools").join("node"))
}

fn get_node_archive_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { return "darwin-arm64"; }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { return "darwin-x64"; }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { return "linux-x64"; }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { return "linux-arm64"; }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { return "win-x64"; }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    { return "unsupported"; }
}

fn get_node_folder_name() -> String {
    format!("node-v{}-{}", NODE_VERSION, get_node_archive_name())
}

pub fn get_local_node_binary() -> Result<Option<PathBuf>, WorkspaceError> {
    let node_dir = get_node_dir()?;
    let folder = get_node_folder_name();

    #[cfg(target_os = "windows")]
    let bin = node_dir.join(&folder).join("node.exe");
    #[cfg(not(target_os = "windows"))]
    let bin = node_dir.join(&folder).join("bin").join("node");

    if bin.exists() {
        Ok(Some(bin))
    } else {
        Ok(None)
    }
}

pub async fn check_system_node() -> Option<PathBuf> {
    // Use a login shell to pick up fnm, nvm, etc.
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let output = Command::new(&shell)
        .arg("-lc")
        .arg("which node")
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().lines().next()?.to_string();
    let path = PathBuf::from(&path);

    if !path.exists() {
        return None;
    }

    // Check version >= 18
    let version_output = Command::new(&path)
        .arg("--version")
        .output()
        .await
        .ok()?;

    let version_str = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
    let major: u32 = version_str.trim_start_matches('v').split('.').next()?.parse().ok()?;

    if major >= 18 {
        Some(path)
    } else {
        None
    }
}

pub fn get_node_binary_path() -> Result<Option<PathBuf>, WorkspaceError> {
    // Check local first
    if let Some(local) = get_local_node_binary()? {
        return Ok(Some(local));
    }
    Ok(None)
}

pub fn get_npm_binary_path() -> Result<Option<PathBuf>, WorkspaceError> {
    let node_dir = get_node_dir()?;
    let folder = get_node_folder_name();

    #[cfg(target_os = "windows")]
    let bin = node_dir.join(&folder).join("npm.cmd");
    #[cfg(not(target_os = "windows"))]
    let bin = node_dir.join(&folder).join("bin").join("npm");

    if bin.exists() {
        Ok(Some(bin))
    } else {
        Ok(None)
    }
}

pub fn get_npx_binary_path() -> Result<Option<PathBuf>, WorkspaceError> {
    let node_dir = get_node_dir()?;
    let folder = get_node_folder_name();

    #[cfg(target_os = "windows")]
    let bin = node_dir.join(&folder).join("npx.cmd");
    #[cfg(not(target_os = "windows"))]
    let bin = node_dir.join(&folder).join("bin").join("npx");

    if bin.exists() {
        Ok(Some(bin))
    } else {
        Ok(None)
    }
}

pub fn get_node_bin_dir() -> Result<PathBuf, WorkspaceError> {
    let node_dir = get_node_dir()?;
    let folder = get_node_folder_name();

    #[cfg(target_os = "windows")]
    { Ok(node_dir.join(&folder)) }
    #[cfg(not(target_os = "windows"))]
    { Ok(node_dir.join(&folder).join("bin")) }
}

pub async fn download_node(app: &tauri::AppHandle) -> Result<PathBuf, WorkspaceError> {
    let archive_name = get_node_archive_name();
    if archive_name == "unsupported" {
        return Err(WorkspaceError::NodeDownloadError("Unsupported platform".to_string()));
    }

    let node_dir = get_node_dir()?;
    std::fs::create_dir_all(&node_dir)
        .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

    // Check if already downloaded
    if let Some(bin) = get_local_node_binary()? {
        return Ok(bin);
    }

    let ext = if cfg!(target_os = "windows") { "zip" } else if cfg!(target_os = "linux") { "tar.xz" } else { "tar.gz" };
    let filename = format!("node-v{}-{}.{}", NODE_VERSION, archive_name, ext);
    let url = format!("https://nodejs.org/dist/v{}/{}", NODE_VERSION, filename);

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "node",
        "message": "Downloading Node.js...",
        "percent": 0
    }));

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| WorkspaceError::NodeDownloadError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(WorkspaceError::NodeDownloadError(format!("HTTP {}", resp.status())));
    }

    let total_size = resp.content_length().unwrap_or(0);
    let archive_path = node_dir.join(&filename);

    // Stream download to file
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(&archive_path).await
        .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| WorkspaceError::NodeDownloadError(e.to_string()))?;
        file.write_all(&chunk).await
            .map_err(|e| WorkspaceError::IoError(e.to_string()))?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
            let _ = app.emit("workspace-progress", serde_json::json!({
                "step": "node",
                "message": format!("Downloading Node.js... {}%", percent),
                "percent": percent
            }));
        }
    }
    file.flush().await.map_err(|e| WorkspaceError::IoError(e.to_string()))?;
    drop(file);

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "node",
        "message": "Extracting Node.js...",
        "percent": 90
    }));

    // Extract archive
    extract_archive(&archive_path, &node_dir)?;

    // Clean up archive
    let _ = std::fs::remove_file(&archive_path);

    let _ = app.emit("workspace-progress", serde_json::json!({
        "step": "node",
        "message": "Node.js installed",
        "percent": 100
    }));

    get_local_node_binary()?
        .ok_or_else(|| WorkspaceError::NodeDownloadError("Binary not found after extraction".to_string()))
}

fn extract_archive(archive_path: &PathBuf, dest: &PathBuf) -> Result<(), WorkspaceError> {
    #[cfg(not(target_os = "windows"))]
    {
        use flate2::read::GzDecoder;
        use std::fs::File;

        let file = File::open(archive_path)
            .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

        if archive_path.extension().map(|e| e == "xz").unwrap_or(false) {
            // For .tar.xz on Linux, use system tar
            let output = std::process::Command::new("tar")
                .arg("xf")
                .arg(archive_path)
                .arg("-C")
                .arg(dest)
                .output()
                .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

            if !output.status.success() {
                return Err(WorkspaceError::NodeDownloadError(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
        } else {
            // .tar.gz
            let gz = GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);
            archive.unpack(dest)
                .map_err(|e| WorkspaceError::NodeDownloadError(e.to_string()))?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| WorkspaceError::IoError(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| WorkspaceError::NodeDownloadError(e.to_string()))?;
        archive.extract(dest)
            .map_err(|e| WorkspaceError::NodeDownloadError(e.to_string()))?;
    }

    Ok(())
}
