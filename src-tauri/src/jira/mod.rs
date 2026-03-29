pub mod client;
pub mod types;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};

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

async fn refresh_jira_token() -> Result<String, JiraError> {
    let config = get_config().map_err(|e| JiraError::ConfigError(e.to_string()))?;
    let refresh_token = crate::config::get_jira_refresh_token()
        .map_err(|e| JiraError::ConfigError(e.to_string()))?
        .ok_or_else(|| JiraError::AuthError("No refresh token available — please re-authenticate".to_string()))?;
    let client_id = config.jira_client_id.as_deref()
        .ok_or_else(|| JiraError::AuthError("No client ID stored — please re-authenticate".to_string()))?;
    let client_secret = config.jira_client_secret.as_deref()
        .ok_or_else(|| JiraError::AuthError("No client secret stored — please re-authenticate".to_string()))?;

    let http = reqwest::Client::new();
    let resp = http
        .post("https://auth.atlassian.com/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "refresh_token",
            "client_id": client_id,
            "client_secret": client_secret,
            "refresh_token": refresh_token,
        }))
        .send()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(JiraError::AuthError(format!("Token refresh failed: {}", body)));
    }

    let token_resp: AtlassianTokenResponse = resp.json().await
        .map_err(|e| JiraError::ParseError(e.to_string()))?;

    crate::config::save_jira_token(token_resp.access_token.clone())
        .map_err(|e| JiraError::ConfigError(e.to_string()))?;
    if let Some(new_refresh) = &token_resp.refresh_token {
        let _ = crate::config::save_jira_refresh_token(new_refresh);
    }

    Ok(token_resp.access_token)
}

fn build_client(token: &str) -> Result<(JiraClient, String), JiraCommandError> {
    let config = get_config()?;

    if config.jira_url.is_empty() {
        return Err(JiraError::ConfigError("Jira URL not configured".to_string()).into());
    }
    if config.jira_email.is_empty() {
        return Err(JiraError::ConfigError("Jira email not configured".to_string()).into());
    }

    // OAuth tokens must use the Atlassian API gateway, not the direct site URL
    let base_url = if let Some(cloud_id) = &config.jira_cloud_id {
        if token.starts_with("ey") {
            format!("https://api.atlassian.com/ex/jira/{}", cloud_id)
        } else {
            config.jira_url.clone()
        }
    } else {
        config.jira_url.clone()
    };

    let client = JiraClient::new(&base_url, &config.jira_email, token)?;
    Ok((client, base_url))
}

fn get_client() -> Result<JiraClient, JiraCommandError> {
    let token = get_jira_token()?
        .ok_or_else(|| JiraError::ConfigError("No Jira token found. Please configure your Jira PAT.".to_string()))?;
    let (client, _) = build_client(&token)?;
    Ok(client)
}

async fn get_client_with_refresh() -> Result<JiraClient, JiraCommandError> {
    let token = get_jira_token()?
        .ok_or_else(|| JiraError::ConfigError("No Jira token found. Please configure your Jira PAT.".to_string()))?;

    // If it's an OAuth token, try a test request and refresh if 401
    if token.starts_with("ey") {
        let (client, base_url) = build_client(&token)?;

        // Quick test request
        let test = reqwest::Client::new()
            .get(format!("{}/rest/api/3/myself", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await;

        if let Ok(resp) = test {
            if resp.status() == 401 {
                // Token expired — refresh
                let new_token = refresh_jira_token().await?;
                let (client, _) = build_client(&new_token)?;
                return Ok(client);
            }
        }

        Ok(client)
    } else {
        let (client, _) = build_client(&token)?;
        Ok(client)
    }
}

#[tauri::command]
pub async fn get_assigned_issues(custom_jql: Option<String>) -> Result<Vec<SimpleIssue>, JiraCommandError> {
    let client = get_client_with_refresh().await?;
    let response = client.get_assigned_issues(custom_jql.as_deref()).await?;

    let issues: Vec<SimpleIssue> = response.issues.into_iter().map(SimpleIssue::from).collect();
    Ok(issues)
}

#[tauri::command]
pub async fn search_issues(jql: String, max_results: Option<i32>) -> Result<Vec<SimpleIssue>, JiraCommandError> {
    let client = get_client_with_refresh().await?;
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
    let client = get_client_with_refresh().await?;

    let started_dt: DateTime<Utc> = started
        .parse()
        .map_err(|e: chrono::ParseError| JiraError::ConfigError(format!("Invalid date format: {}", e)))?;

    let worklog = CreateWorklogRequest::new(time_spent_seconds, started_dt, comment);
    client.create_worklog(&issue_key, worklog).await?;

    Ok(())
}

#[tauri::command]
pub async fn test_jira_connection() -> Result<String, JiraCommandError> {
    let client = get_client_with_refresh().await?;
    let display_name = client.test_connection().await?;
    Ok(display_name)
}

#[tauri::command]
pub async fn get_issue_status(issue_key: String) -> Result<SimpleIssue, JiraCommandError> {
    let client = get_client_with_refresh().await?;
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

// --- Jira OAuth 2.0 (3LO) ---

#[derive(Default)]
pub struct JiraAuthState {
    session: Mutex<Option<JiraAuthSession>>,
}

struct JiraAuthSession {
    receiver: oneshot::Receiver<Result<JiraOAuthResult, JiraCommandError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraOAuthResult {
    pub display_name: String,
    pub email: String,
    pub site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraAuthStartResponse {
    pub auth_url: String,
}

#[derive(Debug, Deserialize)]
struct AtlassianTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AtlassianResource {
    id: String,
    url: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AtlassianUserResponse {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

fn parse_query_params(path: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    if let Some(query) = path.split('?').nth(1) {
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                params.insert(
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                );
            }
        }
    }
    params
}

#[tauri::command]
pub async fn jira_auth_start(
    state: tauri::State<'_, JiraAuthState>,
    client_id: String,
    client_secret: String,
) -> Result<JiraAuthStartResponse, JiraCommandError> {
    let listener = TcpListener::bind(("127.0.0.1", 17548))
        .await
        .map_err(|e| JiraError::RequestError(format!("Failed to bind callback port 17548: {}", e)))?;

    let redirect_uri = "http://localhost:17548/oauth/callback".to_string();
    let state_param = uuid::Uuid::new_v4().to_string();

    let auth_url = format!(
        "https://auth.atlassian.com/authorize?audience=api.atlassian.com&client_id={}&scope={}&redirect_uri={}&state={}&response_type=code&prompt=consent",
        urlencoding::encode(&client_id),
        urlencoding::encode("read:jira-work read:jira-user write:jira-work offline_access read:me"),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state_param),
    );

    let (sender, receiver) = oneshot::channel();

    let cid = client_id.clone();
    let csec = client_secret.clone();
    let ruri = redirect_uri.clone();
    let expected_state = state_param.clone();

    tokio::spawn(async move {
        let result = handle_jira_callback(listener, cid, csec, ruri, expected_state).await;
        let _ = sender.send(result);
    });

    let mut session = state.session.lock().await;
    *session = Some(JiraAuthSession { receiver });

    Ok(JiraAuthStartResponse { auth_url })
}

async fn handle_jira_callback(
    listener: TcpListener,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    expected_state: String,
) -> Result<JiraOAuthResult, JiraCommandError> {
    let (mut socket, _) = listener
        .accept()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    let mut buffer = [0u8; 4096];
    let bytes_read = socket
        .read(&mut buffer)
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string();

    // Send response HTML
    let html = "<html><head><title>Mira</title></head><body><h2>Jira Connected</h2><p>You can close this window and return to the app.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = socket.write_all(response.as_bytes()).await;

    let params = parse_query_params(&path);

    if let Some(error) = params.get("error") {
        return Err(JiraError::AuthError(format!("Auth error: {}", error)).into());
    }

    let code = params
        .get("code")
        .ok_or_else(|| JiraError::AuthError("No authorization code received".to_string()))?;

    let state = params
        .get("state")
        .ok_or_else(|| JiraError::AuthError("No state parameter".to_string()))?;

    if *state != expected_state {
        return Err(JiraError::AuthError("State mismatch — possible CSRF".to_string()).into());
    }

    // Exchange code for token
    let http = reqwest::Client::new();
    let token_resp = http
        .post("https://auth.atlassian.com/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    let token_status = token_resp.status();
    let token_body = token_resp
        .text()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    if !token_status.is_success() {
        return Err(JiraError::ApiError {
            status: token_status.as_u16(),
            message: token_body,
        }
        .into());
    }

    let token: AtlassianTokenResponse = serde_json::from_str(&token_body)
        .map_err(|e| JiraError::ParseError(format!("{}: {}", e, token_body)))?;

    // Get accessible resources (cloud ID and site URL)
    let resources_resp = http
        .get("https://api.atlassian.com/oauth/token/accessible-resources")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    let resources: Vec<AtlassianResource> = resources_resp
        .json()
        .await
        .map_err(|e| JiraError::ParseError(e.to_string()))?;

    let resource = resources
        .into_iter()
        .next()
        .ok_or_else(|| JiraError::AuthError("No Jira sites found for this account".to_string()))?;

    // Get user info via Jira API
    let user_resp = http
        .get(format!(
            "https://api.atlassian.com/ex/jira/{}/rest/api/3/myself",
            resource.id
        ))
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| JiraError::RequestError(e.to_string()))?;

    let user: AtlassianUserResponse = user_resp
        .json()
        .await
        .map_err(|e| JiraError::ParseError(e.to_string()))?;

    let display_name = user.display_name.unwrap_or_else(|| "Unknown".to_string());
    let email = user
        .email_address
        .unwrap_or_else(|| "unknown@email.com".to_string());

    // Save tokens
    crate::config::save_jira_token(token.access_token)?;
    if let Some(refresh) = &token.refresh_token {
        crate::config::save_jira_refresh_token(refresh)?;
    }

    let site_url = resource.url.trim_end_matches('/').to_string();
    let mut config = get_config()?;
    config.jira_url = site_url.clone();
    config.jira_cloud_id = Some(resource.id.clone());
    config.jira_client_id = Some(client_id.clone());
    config.jira_client_secret = Some(client_secret.clone());
    config.jira_email = email.clone();
    crate::config::save_config(config)?;

    Ok(JiraOAuthResult {
        display_name,
        email,
        site_url,
    })
}

#[tauri::command]
pub async fn jira_auth_wait(
    state: tauri::State<'_, JiraAuthState>,
) -> Result<JiraOAuthResult, JiraCommandError> {
    let receiver = {
        let mut session = state.session.lock().await;
        session
            .take()
            .ok_or_else(|| JiraError::AuthError("No auth session in progress".to_string()))?
            .receiver
    };

    let result = timeout(Duration::from_secs(180), receiver)
        .await
        .map_err(|_| JiraError::AuthError("Auth timed out (180s)".to_string()))?
        .map_err(|_| JiraError::AuthError("Auth session was cancelled".to_string()))?;

    result
}
