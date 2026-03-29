use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

const SERVICE_NAME: &str = "com.mira.app";
const JIRA_TOKEN_KEY: &str = "jira-pat";
const TOKEN_FILE_NAME: &str = ".token";
const GITHUB_TOKEN_FILE_NAME: &str = ".github_token";
const JIRA_REFRESH_TOKEN_FILE_NAME: &str = ".jira_refresh_token";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    #[error("Failed to write config file: {0}")]
    WriteError(String),
    #[error("Failed to access keyring: {0}")]
    KeyringError(String),
    #[error("Configuration directory not found")]
    ConfigDirNotFound,
}

impl serde::Serialize for ConfigError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub jira_url: String,
    pub jira_email: String,
    pub jira_cloud_id: Option<String>,
    pub jira_client_id: Option<String>,
    pub jira_client_secret: Option<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub selected_calendar: Option<String>,
    pub sync_frequency: SyncFrequency,
    pub auto_sync_on_launch: bool,
    pub jql_filter: Option<String>,
    pub event_title_template: String,
    pub timezone: Option<String>,
    pub default_event_color: Option<String>,
    // GitHub configuration
    pub github_username: String,
    pub pr_event_title_template: String,
    pub pr_default_event_color: Option<String>,
    // Per-calendar color index (calendar UID -> index into EVENT_COLORS)
    pub calendar_colors: std::collections::HashMap<String, u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SyncFrequency {
    #[default]
    Manual,
    Hourly,
    Daily,
}

fn get_config_dir() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir().ok_or(ConfigError::ConfigDirNotFound)?;
    let app_config_dir = config_dir.join("mira");
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
    }
    Ok(app_config_dir)
}

fn get_config_path() -> Result<PathBuf, ConfigError> {
    Ok(get_config_dir()?.join("config.json"))
}

fn get_token_file_path() -> Result<PathBuf, ConfigError> {
    Ok(get_config_dir()?.join(TOKEN_FILE_NAME))
}

fn get_github_token_file_path() -> Result<PathBuf, ConfigError> {
    Ok(get_config_dir()?.join(GITHUB_TOKEN_FILE_NAME))
}

fn delete_token_from_keyring() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, JIRA_TOKEN_KEY)
        .map_err(|e| e.to_string())?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// File-based fallback for token storage
fn save_token_to_file(token: &str) -> Result<(), ConfigError> {
    let token_path = get_token_file_path()?;
    fs::write(&token_path, token)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;

    // Try to set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(&token_path, permissions);
    }

    Ok(())
}

fn get_token_from_file() -> Result<Option<String>, ConfigError> {
    let token_path = get_token_file_path()?;
    if !token_path.exists() {
        return Ok(None);
    }

    let token = fs::read_to_string(&token_path)
        .map_err(|e| ConfigError::ReadError(e.to_string()))?;

    let token = token.trim();
    if token.is_empty() {
        Ok(None)
    } else {
        Ok(Some(token.to_string()))
    }
}

fn delete_token_file() -> Result<(), ConfigError> {
    let token_path = get_token_file_path()?;
    if token_path.exists() {
        fs::remove_file(&token_path)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_config() -> Result<AppConfig, ConfigError> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let default_config = AppConfig {
            event_title_template: "[{key}] {summary}".to_string(),
            pr_event_title_template: "[PR Review] {repo}: {title}".to_string(),
            ..Default::default()
        };
        return Ok(default_config);
    }

    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| ConfigError::ReadError(e.to_string()))?;

    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| ConfigError::ReadError(e.to_string()))?;

    Ok(config)
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path()?;

    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;

    fs::write(&config_path, config_str)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn save_jira_token(token: String) -> Result<(), ConfigError> {
    log::info!("save_jira_token called, token length: {}", token.len());

    // Always use file storage for reliability on macOS
    // Keyring can silently fail without proper entitlements
    match save_token_to_file(&token) {
        Ok(()) => {
            log::info!("Token saved to file successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to save token to file: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_jira_token() -> Result<Option<String>, ConfigError> {
    log::info!("get_jira_token called");

    // Use file-based storage
    log::info!("Checking file-based token storage");
    match get_token_from_file() {
        Ok(Some(token)) => {
            log::info!("Token found in file, length: {}", token.len());
            Ok(Some(token))
        }
        Ok(None) => {
            log::info!("No token in file");
            Ok(None)
        }
        Err(e) => {
            log::error!("Failed to read token from file: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn delete_jira_token() -> Result<(), ConfigError> {
    // Try to delete from both places
    let keyring_result = delete_token_from_keyring();
    let file_result = delete_token_file();

    // Return error only if both fail
    if let Err(e) = keyring_result {
        log::debug!("Keyring delete failed: {}", e);
    }

    file_result
}

#[tauri::command]
pub fn has_jira_token() -> Result<bool, ConfigError> {
    // Check file-based storage
    match get_token_from_file()? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

// Jira refresh token management

pub fn save_jira_refresh_token(token: &str) -> Result<(), ConfigError> {
    let path = get_config_dir()?.join(JIRA_REFRESH_TOKEN_FILE_NAME);
    fs::write(&path, token).map_err(|e| ConfigError::WriteError(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn get_jira_refresh_token() -> Result<Option<String>, ConfigError> {
    let path = get_config_dir()?.join(JIRA_REFRESH_TOKEN_FILE_NAME);
    if !path.exists() {
        return Ok(None);
    }
    let token = fs::read_to_string(&path).map_err(|e| ConfigError::ReadError(e.to_string()))?;
    let token = token.trim();
    if token.is_empty() { Ok(None) } else { Ok(Some(token.to_string())) }
}

// GitHub token management functions

fn save_github_token_to_file(token: &str) -> Result<(), ConfigError> {
    let token_path = get_github_token_file_path()?;
    fs::write(&token_path, token)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;

    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(&token_path, permissions);
    }

    Ok(())
}

fn get_github_token_from_file() -> Result<Option<String>, ConfigError> {
    let token_path = get_github_token_file_path()?;
    if !token_path.exists() {
        return Ok(None);
    }

    let token = fs::read_to_string(&token_path)
        .map_err(|e| ConfigError::ReadError(e.to_string()))?;

    let token = token.trim();
    if token.is_empty() {
        Ok(None)
    } else {
        Ok(Some(token.to_string()))
    }
}

fn delete_github_token_file() -> Result<(), ConfigError> {
    let token_path = get_github_token_file_path()?;
    if token_path.exists() {
        fs::remove_file(&token_path)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_github_token(token: String) -> Result<(), ConfigError> {
    log::info!("save_github_token called, token length: {}", token.len());

    match save_github_token_to_file(&token) {
        Ok(()) => {
            log::info!("GitHub token saved to file successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to save GitHub token to file: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_github_token() -> Result<Option<String>, ConfigError> {
    log::info!("get_github_token called");

    log::info!("Checking file-based GitHub token storage");
    match get_github_token_from_file() {
        Ok(Some(token)) => {
            log::info!("GitHub token found in file, length: {}", token.len());
            Ok(Some(token))
        }
        Ok(None) => {
            log::info!("No GitHub token in file");
            Ok(None)
        }
        Err(e) => {
            log::error!("Failed to read GitHub token from file: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn delete_github_token() -> Result<(), ConfigError> {
    delete_github_token_file()
}

#[tauri::command]
pub fn has_github_token() -> Result<bool, ConfigError> {
    match get_github_token_from_file()? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
