pub mod client;
pub mod error;
pub mod types;

use crate::config::{get_config, ConfigError};
use client::GitHubClient;
use error::GitHubError;
use types::SimplePullRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum GitHubCommandError {
    Config(ConfigError),
    GitHub(GitHubError),
}

impl std::fmt::Display for GitHubCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitHubCommandError::Config(e) => write!(f, "Configuration error: {}", e),
            GitHubCommandError::GitHub(e) => write!(f, "GitHub error: {}", e),
        }
    }
}

impl serde::Serialize for GitHubCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<ConfigError> for GitHubCommandError {
    fn from(e: ConfigError) -> Self {
        GitHubCommandError::Config(e)
    }
}

impl From<GitHubError> for GitHubCommandError {
    fn from(e: GitHubError) -> Self {
        GitHubCommandError::GitHub(e)
    }
}

fn get_client() -> Result<GitHubClient, GitHubCommandError> {
    let token = crate::config::get_github_token()?
        .ok_or_else(|| GitHubError::Config("No GitHub token found. Please configure your GitHub PAT.".to_string()))?;

    let client = GitHubClient::new(&token)?;
    Ok(client)
}

#[tauri::command]
pub async fn get_pull_requests() -> Result<Vec<SimplePullRequest>, GitHubCommandError> {
    let client = get_client()?;
    let config = get_config()?;

    // Get the username - either from config or by fetching from API
    let username = if !config.github_username.is_empty() {
        config.github_username
    } else {
        // Fetch username from API
        let user = client.get_user().await?;
        user.login
    };

    // Fetch both review-requested and authored PRs concurrently
    let (review_result, authored_result) = tokio::join!(
        client.get_review_requests(&username),
        client.get_authored_prs(&username)
    );

    let review_prs = review_result?;
    let authored_prs = authored_result?;

    // Merge and deduplicate by PR id; authored takes priority
    let mut seen = std::collections::HashMap::new();
    for pr in authored_prs {
        seen.insert(pr.id, pr);
    }
    for pr in review_prs {
        seen.entry(pr.id).or_insert(pr);
    }

    let mut prs: Vec<SimplePullRequest> = seen.into_values().collect();
    prs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(prs)
}

#[tauri::command]
pub async fn test_github_connection() -> Result<String, GitHubCommandError> {
    let client = get_client()?;
    let display_name = client.test_connection().await?;
    Ok(display_name)
}

// Token management commands are in config/mod.rs
// Re-export them here for convenience
pub use crate::config::{
    delete_github_token,
    get_github_token,
    has_github_token,
    save_github_token,
};

// --- GitHub Device Flow OAuth ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFlowTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[tauri::command]
pub async fn github_device_flow_start(
    client_id: String,
) -> Result<DeviceCodeResponse, GitHubCommandError> {
    let http = reqwest::Client::new();

    let response = http
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("scope", "repo read:user"),
        ])
        .send()
        .await
        .map_err(|e| GitHubError::Request(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| GitHubError::Request(e.to_string()))?;

    if !status.is_success() {
        return Err(GitHubError::Api {
            status: status.as_u16(),
            message: body,
        }
        .into());
    }

    let device_code: DeviceCodeResponse = serde_json::from_str(&body)
        .map_err(|e| GitHubError::Parse(format!("{}: {}", e, body)))?;

    Ok(device_code)
}

#[tauri::command]
pub async fn github_device_flow_poll(
    client_id: String,
    device_code: String,
) -> Result<String, GitHubCommandError> {
    let http = reqwest::Client::new();

    let response = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("device_code", device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| GitHubError::Request(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| GitHubError::Request(e.to_string()))?;

    if !status.is_success() {
        return Err(GitHubError::Api {
            status: status.as_u16(),
            message: body,
        }
        .into());
    }

    // GitHub returns errors in the JSON body even with 200 status
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| GitHubError::Parse(e.to_string()))?;

    if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
        match error {
            "authorization_pending" => {
                return Err(GitHubError::Auth("authorization_pending".to_string()).into());
            }
            "slow_down" => {
                return Err(GitHubError::Auth("slow_down".to_string()).into());
            }
            "expired_token" => {
                return Err(GitHubError::Auth("Device code expired. Please try again.".to_string()).into());
            }
            "access_denied" => {
                return Err(GitHubError::Auth("Access denied by user.".to_string()).into());
            }
            other => {
                let desc = json
                    .get("error_description")
                    .and_then(|d| d.as_str())
                    .unwrap_or(other);
                return Err(GitHubError::Auth(desc.to_string()).into());
            }
        }
    }

    let access_token = json
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| GitHubError::Parse("No access_token in response".to_string()))?;

    // Save the token
    crate::config::save_github_token(access_token.to_string())?;

    // Fetch username and save to config
    let client = GitHubClient::new(access_token)?;
    let user = client.get_user().await?;
    let mut config = get_config()?;
    config.github_username = user.login.clone();
    crate::config::save_config(config)?;

    Ok(user.name.unwrap_or(user.login))
}
