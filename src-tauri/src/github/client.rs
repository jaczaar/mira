use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Client;

use super::error::GitHubError;
use super::types::{GitHubAccountInfo, PullRequest, SearchResponse, SimplePullRequest};

pub struct GitHubClient {
    client: Client,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self, GitHubError> {
        let mut headers = HeaderMap::new();

        // GitHub requires Bearer token auth
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| GitHubError::Auth(e.to_string()))?,
        );

        // GitHub API requires Accept header
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        // GitHub API requires User-Agent
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mira-App/1.0"),
        );

        // GitHub API version header
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        Ok(Self {
            client,
            base_url: "https://api.github.com".to_string(),
        })
    }

    /// Get the authenticated user's info
    pub async fn get_user(&self) -> Result<GitHubAccountInfo, GitHubError> {
        let url = format!("{}/user", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GitHubError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let user: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitHubError::Parse(e.to_string()))?;

        Ok(GitHubAccountInfo {
            login: user["login"].as_str().unwrap_or("unknown").to_string(),
            name: user["name"].as_str().map(|s| s.to_string()),
            avatar_url: user["avatar_url"].as_str().map(|s| s.to_string()),
        })
    }

    /// Test the connection by fetching user info
    pub async fn test_connection(&self) -> Result<String, GitHubError> {
        let user = self.get_user().await?;
        Ok(user.name.unwrap_or(user.login))
    }

    /// Get pull requests where the user is a requested reviewer
    pub async fn get_review_requests(&self, username: &str) -> Result<Vec<SimplePullRequest>, GitHubError> {
        // Use GitHub search API to find PRs where user is requested reviewer
        // Query: is:pr is:open review-requested:{username} archived:false
        let query = format!(
            "is:pr is:open review-requested:{} archived:false",
            username
        );
        let encoded_query = urlencoding::encode(&query);
        let url = format!(
            "{}/search/issues?q={}&sort=updated&order=desc&per_page=100",
            self.base_url, encoded_query
        );

        log::info!("Fetching review requests: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GitHubError::Api {
                status: status.as_u16(),
                message: response_text,
            });
        }

        log::info!("Search response (first 500 chars): {}", &response_text[..response_text.len().min(500)]);

        let search_result: SearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to parse GitHub response: {}. Response: {}", e, &response_text[..response_text.len().min(1000)]);
                GitHubError::Parse(format!("{} - Response preview: {}", e, &response_text[..response_text.len().min(200)]))
            })?;

        // For each PR, fetch full details to get branch info
        let mut prs = Vec::new();
        for item in search_result.items {
            // Try to get full PR details for branch info
            let pr_details = self.get_pull_request_details(&item.repository_url, item.number).await.ok();
            prs.push(SimplePullRequest::from_search_item(item, pr_details.as_ref(), "reviewer"));
        }

        Ok(prs)
    }

    /// Get pull requests authored by the user
    pub async fn get_authored_prs(&self, username: &str) -> Result<Vec<SimplePullRequest>, GitHubError> {
        let query = format!(
            "is:pr is:open author:{} archived:false",
            username
        );
        let encoded_query = urlencoding::encode(&query);
        let url = format!(
            "{}/search/issues?q={}&sort=updated&order=desc&per_page=100",
            self.base_url, encoded_query
        );

        log::info!("Fetching authored PRs: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GitHubError::Api {
                status: status.as_u16(),
                message: response_text,
            });
        }

        let search_result: SearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to parse GitHub response: {}. Response: {}", e, &response_text[..response_text.len().min(1000)]);
                GitHubError::Parse(format!("{} - Response preview: {}", e, &response_text[..response_text.len().min(200)]))
            })?;

        let mut prs = Vec::new();
        for item in search_result.items {
            let pr_details = self.get_pull_request_details(&item.repository_url, item.number).await.ok();
            prs.push(SimplePullRequest::from_search_item(item, pr_details.as_ref(), "author"));
        }

        Ok(prs)
    }

    /// Get full PR details including branch info
    async fn get_pull_request_details(&self, repo_url: &str, pr_number: i64) -> Result<PullRequest, GitHubError> {
        // repo_url is like "https://api.github.com/repos/owner/repo"
        let url = format!("{}/pulls/{}", repo_url, pr_number);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GitHubError::Request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GitHubError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let pr: PullRequest = response
            .json()
            .await
            .map_err(|e| GitHubError::Parse(e.to_string()))?;

        Ok(pr)
    }
}
