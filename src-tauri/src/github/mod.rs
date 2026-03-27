pub mod client;
pub mod error;
pub mod types;

use crate::config::{get_config, ConfigError};
use client::GitHubClient;
use error::GitHubError;
use types::SimplePullRequest;

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
