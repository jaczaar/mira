use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use thiserror::Error;

use super::types::{CreateWorklogRequest, JiraSearchResponse, JiraWorklog};

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("HTTP request failed: {0}")]
    RequestError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Jira API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl serde::Serialize for JiraError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct JiraClient {
    client: Client,
    base_url: String,
}

impl JiraClient {
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Result<Self, JiraError> {
        let mut headers = HeaderMap::new();

        // Create Basic Auth header
        let auth_string = format!("{}:{}", email, api_token);
        let encoded = base64_encode(&auth_string);
        let auth_value = format!("Basic {}", encoded);

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| JiraError::AuthError(e.to_string()))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| JiraError::RequestError(e.to_string()))?;

        // Normalize base URL (remove trailing slash)
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self { client, base_url })
    }

    pub async fn search_issues(&self, jql: &str, max_results: i32) -> Result<JiraSearchResponse, JiraError> {
        // Use the new /rest/api/3/search/jql endpoint (old /search is deprecated)
        let url = format!("{}/rest/api/3/search/jql", self.base_url);

        let fields = [
            "summary",
            "description",
            "status",
            "priority",
            "project",
            "assignee",
            "reporter",
            "timeoriginalestimate",
            "timeestimate",
            "timespent",
            "created",
            "updated",
            "duedate",
            "issuetype",
            "labels",
            "parent"
        ].join(",");

        let encoded_jql = urlencoding::encode(jql);
        let query_url = format!("{}?jql={}&maxResults={}&fields={}", url, encoded_jql, max_results, fields);

        let response = self
            .client
            .get(&query_url)
            .send()
            .await
            .map_err(|e| JiraError::RequestError(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| JiraError::RequestError(e.to_string()))?;

        if !status.is_success() {
            return Err(JiraError::ApiError {
                status: status.as_u16(),
                message: response_text,
            });
        }

        log::info!("Jira search response (first 500 chars): {}", &response_text[..response_text.len().min(500)]);

        let search_response: JiraSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to parse Jira response: {}. Response: {}", e, &response_text[..response_text.len().min(1000)]);
                JiraError::ParseError(format!("{} - Response preview: {}", e, &response_text[..response_text.len().min(200)]))
            })?;

        Ok(search_response)
    }

    pub async fn get_assigned_issues(&self, custom_jql: Option<&str>) -> Result<JiraSearchResponse, JiraError> {
        let jql = custom_jql.unwrap_or("assignee = currentUser() AND resolution = Unresolved ORDER BY priority DESC, updated DESC");
        self.search_issues(jql, 50).await
    }

    pub async fn create_worklog(
        &self,
        issue_key: &str,
        worklog: CreateWorklogRequest,
    ) -> Result<JiraWorklog, JiraError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/worklog",
            self.base_url, issue_key
        );

        let response = self
            .client
            .post(&url)
            .json(&worklog)
            .send()
            .await
            .map_err(|e| JiraError::RequestError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JiraError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let worklog_response: JiraWorklog = response
            .json()
            .await
            .map_err(|e| JiraError::ParseError(e.to_string()))?;

        Ok(worklog_response)
    }

    pub async fn test_connection(&self) -> Result<String, JiraError> {
        let url = format!("{}/rest/api/3/myself", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JiraError::RequestError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JiraError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| JiraError::ParseError(e.to_string()))?;

        let display_name = user["displayName"]
            .as_str()
            .unwrap_or("Unknown User")
            .to_string();

        Ok(display_name)
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut n: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            n |= (byte as u32) << (16 - i * 8);
        }

        let padding = 3 - chunk.len();
        for i in 0..(4 - padding) {
            let idx = ((n >> (18 - i * 6)) & 0x3F) as usize;
            result.push(ALPHABET[idx] as char);
        }

        for _ in 0..padding {
            result.push('=');
        }
    }

    result
}
