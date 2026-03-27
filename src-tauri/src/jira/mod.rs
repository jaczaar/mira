pub mod client;
pub mod types;

use chrono::{DateTime, Utc};

use crate::config::{get_config, get_jira_token, ConfigError};
use client::{JiraClient, JiraError};
use types::{CreateWorklogRequest, SimpleIssue};

#[derive(Debug)]
pub enum JiraCommandError {
    Config(ConfigError),
    Jira(JiraError),
}

impl std::fmt::Display for JiraCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JiraCommandError::Config(e) => write!(f, "Configuration error: {}", e),
            JiraCommandError::Jira(e) => write!(f, "Jira error: {}", e),
        }
    }
}

impl serde::Serialize for JiraCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<ConfigError> for JiraCommandError {
    fn from(e: ConfigError) -> Self {
        JiraCommandError::Config(e)
    }
}

impl From<JiraError> for JiraCommandError {
    fn from(e: JiraError) -> Self {
        JiraCommandError::Jira(e)
    }
}

fn get_client() -> Result<JiraClient, JiraCommandError> {
    let config = get_config()?;
    let token = get_jira_token()?
        .ok_or_else(|| JiraError::ConfigError("No Jira token found. Please configure your Jira PAT.".to_string()))?;

    if config.jira_url.is_empty() {
        return Err(JiraError::ConfigError("Jira URL not configured".to_string()).into());
    }
    if config.jira_email.is_empty() {
        return Err(JiraError::ConfigError("Jira email not configured".to_string()).into());
    }

    let client = JiraClient::new(&config.jira_url, &config.jira_email, &token)?;
    Ok(client)
}

#[tauri::command]
pub async fn get_assigned_issues(custom_jql: Option<String>) -> Result<Vec<SimpleIssue>, JiraCommandError> {
    let client = get_client()?;
    let response = client.get_assigned_issues(custom_jql.as_deref()).await?;

    let issues: Vec<SimpleIssue> = response.issues.into_iter().map(SimpleIssue::from).collect();
    Ok(issues)
}

#[tauri::command]
pub async fn search_issues(jql: String, max_results: Option<i32>) -> Result<Vec<SimpleIssue>, JiraCommandError> {
    let client = get_client()?;
    let response = client.search_issues(&jql, max_results.unwrap_or(50)).await?;

    let issues: Vec<SimpleIssue> = response.issues.into_iter().map(SimpleIssue::from).collect();
    Ok(issues)
}

#[tauri::command]
pub async fn create_worklog(
    issue_key: String,
    time_spent_seconds: i64,
    started: String,
    comment: Option<String>,
) -> Result<(), JiraCommandError> {
    let client = get_client()?;

    let started_dt: DateTime<Utc> = started
        .parse()
        .map_err(|e: chrono::ParseError| JiraError::ConfigError(format!("Invalid date format: {}", e)))?;

    let worklog = CreateWorklogRequest::new(time_spent_seconds, started_dt, comment);
    client.create_worklog(&issue_key, worklog).await?;

    Ok(())
}

#[tauri::command]
pub async fn test_jira_connection() -> Result<String, JiraCommandError> {
    let client = get_client()?;
    let display_name = client.test_connection().await?;
    Ok(display_name)
}

#[tauri::command]
pub async fn get_issue_status(issue_key: String) -> Result<SimpleIssue, JiraCommandError> {
    let client = get_client()?;
    let jql = format!("key = {}", issue_key);
    let response = client.search_issues(&jql, 1).await?;

    response
        .issues
        .into_iter()
        .next()
        .map(SimpleIssue::from)
        .ok_or_else(|| JiraError::ApiError {
            status: 404,
            message: format!("Issue {} not found", issue_key),
        }.into())
}
