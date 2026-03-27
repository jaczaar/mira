use std::fs;
use std::path::PathBuf;

use super::error::GoogleError;
use super::types::GoogleToken;

const TOKEN_FILE_NAME: &str = "google_token.json";

fn get_config_dir() -> Result<PathBuf, GoogleError> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        GoogleError::Config("Configuration directory not found".to_string())
    })?;
    let app_config_dir = config_dir.join("mira");
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .map_err(|e| GoogleError::Io(e.to_string()))?;
    }
    Ok(app_config_dir)
}

fn get_token_path() -> Result<PathBuf, GoogleError> {
    Ok(get_config_dir()?.join(TOKEN_FILE_NAME))
}

pub fn load_token() -> Result<Option<GoogleToken>, GoogleError> {
    let token_path = get_token_path()?;
    if !token_path.exists() {
        return Ok(None);
    }

    let token_str = fs::read_to_string(&token_path)
        .map_err(|e| GoogleError::Io(e.to_string()))?;

    let token: GoogleToken = serde_json::from_str(&token_str)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;

    Ok(Some(token))
}

pub fn save_token(token: &GoogleToken) -> Result<(), GoogleError> {
    let token_path = get_token_path()?;
    let token_str = serde_json::to_string_pretty(token)
        .map_err(|e| GoogleError::Parse(e.to_string()))?;

    fs::write(&token_path, token_str)
        .map_err(|e| GoogleError::Io(e.to_string()))?;

    Ok(())
}

pub fn delete_token() -> Result<(), GoogleError> {
    let token_path = get_token_path()?;
    if token_path.exists() {
        fs::remove_file(&token_path)
            .map_err(|e| GoogleError::Io(e.to_string()))?;
    }
    Ok(())
}
