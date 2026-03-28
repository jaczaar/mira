use std::fs;
use std::path::PathBuf;

use super::error::GoogleError;
use super::types::GoogleToken;

fn get_tokens_dir() -> Result<PathBuf, GoogleError> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        GoogleError::Config("Configuration directory not found".to_string())
    })?;
    let tokens_dir = config_dir.join("mira").join("google_tokens");
    if !tokens_dir.exists() {
        fs::create_dir_all(&tokens_dir)
            .map_err(|e| GoogleError::Io(e.to_string()))?;
    }
    Ok(tokens_dir)
}

fn get_legacy_token_path() -> Result<PathBuf, GoogleError> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        GoogleError::Config("Configuration directory not found".to_string())
    })?;
    Ok(config_dir.join("mira").join("google_token.json"))
}

fn sanitize_email(email: &str) -> String {
    email.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn get_token_path_for(email: &str) -> Result<PathBuf, GoogleError> {
    let filename = format!("{}.json", sanitize_email(email));
    Ok(get_tokens_dir()?.join(filename))
}

pub fn load_token_for(email: &str) -> Result<Option<GoogleToken>, GoogleError> {
    let token_path = get_token_path_for(email)?;
    if !token_path.exists() {
        return Ok(None);
    }
    let token_str = fs::read_to_string(&token_path)
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    let token: GoogleToken = serde_json::from_str(&token_str)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;
    Ok(Some(token))
}

pub fn save_token_for(email: &str, token: &GoogleToken) -> Result<(), GoogleError> {
    let token_path = get_token_path_for(email)?;
    let token_str = serde_json::to_string_pretty(token)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;
    fs::write(&token_path, token_str)
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    Ok(())
}

pub fn delete_token_for(email: &str) -> Result<(), GoogleError> {
    let token_path = get_token_path_for(email)?;
    if token_path.exists() {
        fs::remove_file(&token_path)
            .map_err(|e| GoogleError::Io(e.to_string()))?;
    }
    Ok(())
}

pub fn list_accounts() -> Result<Vec<String>, GoogleError> {
    let tokens_dir = get_tokens_dir()?;
    let mut accounts = Vec::new();
    if tokens_dir.exists() {
        for entry in fs::read_dir(&tokens_dir).map_err(|e| GoogleError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| GoogleError::Io(e.to_string()))?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    accounts.push(stem.to_string());
                }
            }
        }
    }
    Ok(accounts)
}

/// Migrate legacy single-token file to the new per-account structure.
/// Returns the email if migration happened.
pub fn migrate_legacy_token(_client: &super::client::GoogleClient) -> Result<Option<String>, GoogleError> {
    let legacy_path = get_legacy_token_path()?;
    if !legacy_path.exists() {
        return Ok(None);
    }
    let token_str = fs::read_to_string(&legacy_path)
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    let token: GoogleToken = serde_json::from_str(&token_str)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;

    // We need the email to save under the right filename.
    // We'll save temporarily and let the caller resolve + re-save after fetching user info.
    // For now, save under a temp name and return the token so caller can get the email.
    // Actually, let's just return None here and handle migration in auth_status where we have the client.
    // We'll keep the legacy file around until it's migrated.
    let _ = token; // suppress warning
    Ok(None)
}

// Legacy compat: load the old single token file (used during migration)
pub fn load_legacy_token() -> Result<Option<GoogleToken>, GoogleError> {
    let legacy_path = get_legacy_token_path()?;
    if !legacy_path.exists() {
        return Ok(None);
    }
    let token_str = fs::read_to_string(&legacy_path)
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    let token: GoogleToken = serde_json::from_str(&token_str)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;
    Ok(Some(token))
}

pub fn delete_legacy_token() -> Result<(), GoogleError> {
    let legacy_path = get_legacy_token_path()?;
    if legacy_path.exists() {
        fs::remove_file(&legacy_path)
            .map_err(|e| GoogleError::Io(e.to_string()))?;
    }
    Ok(())
}
