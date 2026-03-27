use serde::{Deserialize, Serialize};

/// GitHub user from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: i64,
    pub avatar_url: Option<String>,
    #[serde(rename = "html_url")]
    pub html_url: Option<String>,
}

/// GitHub repository reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub user: Option<GitHubUser>,
    pub repo: Option<Repository>,
}

/// GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub private: bool,
}

/// Pull request from GitHub API (search results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestSearchItem {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub user: Option<GitHubUser>,
    pub draft: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
    pub pull_request: Option<PullRequestUrls>,
    pub repository_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestUrls {
    pub url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
}

/// Full pull request details from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub user: Option<GitHubUser>,
    pub draft: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
    pub head: GitHubRef,
    pub base: GitHubRef,
}

/// Search response from GitHub API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_count: i64,
    pub incomplete_results: bool,
    pub items: Vec<PullRequestSearchItem>,
}

/// Simplified pull request for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub url: String,
    pub repo_name: String,
    pub repo_full_name: String,
    pub author: String,
    pub author_avatar: Option<String>,
    pub branch: String,
    pub target_branch: String,
    pub state: String,
    pub is_draft: bool,
    pub created_at: String,
    pub updated_at: String,
    pub jira_key: Option<String>,
    pub pr_role: String, // "reviewer" or "author"
}

impl SimplePullRequest {
    pub fn from_search_item(item: PullRequestSearchItem, pr_details: Option<&PullRequest>, role: &str) -> Self {
        // Extract repo name from repository_url
        // e.g., "https://api.github.com/repos/owner/repo" -> "repo" and "owner/repo"
        let repo_parts: Vec<&str> = item.repository_url.split('/').collect();
        let repo_name = repo_parts.last().unwrap_or(&"unknown").to_string();
        let repo_full_name = if repo_parts.len() >= 2 {
            format!("{}/{}", repo_parts[repo_parts.len() - 2], repo_parts[repo_parts.len() - 1])
        } else {
            repo_name.clone()
        };

        let author = item.user.as_ref().map(|u| u.login.clone()).unwrap_or_else(|| "unknown".to_string());
        let author_avatar = item.user.as_ref().and_then(|u| u.avatar_url.clone());

        // Get branch info from PR details if available
        let (branch, target_branch) = if let Some(pr) = pr_details {
            (pr.head.ref_name.clone(), pr.base.ref_name.clone())
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };

        // Extract Jira key from title or branch
        let jira_key = extract_jira_key(&item.title, &branch);

        SimplePullRequest {
            id: item.id,
            number: item.number,
            title: item.title,
            url: item.html_url,
            repo_name,
            repo_full_name,
            author,
            author_avatar,
            branch,
            target_branch,
            state: item.state,
            is_draft: item.draft.unwrap_or(false),
            created_at: item.created_at,
            updated_at: item.updated_at,
            jira_key,
            pr_role: role.to_string(),
        }
    }

    pub fn from_pull_request(pr: PullRequest, role: &str) -> Self {
        let repo_full_name = pr.head.repo.as_ref()
            .map(|r| r.full_name.clone())
            .unwrap_or_else(|| "unknown/unknown".to_string());
        let repo_name = pr.head.repo.as_ref()
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let author = pr.user.as_ref().map(|u| u.login.clone()).unwrap_or_else(|| "unknown".to_string());
        let author_avatar = pr.user.as_ref().and_then(|u| u.avatar_url.clone());

        let jira_key = extract_jira_key(&pr.title, &pr.head.ref_name);

        SimplePullRequest {
            id: pr.id,
            number: pr.number,
            title: pr.title,
            url: pr.html_url,
            repo_name,
            repo_full_name,
            author,
            author_avatar,
            branch: pr.head.ref_name,
            target_branch: pr.base.ref_name,
            state: pr.state,
            is_draft: pr.draft.unwrap_or(false),
            created_at: pr.created_at,
            updated_at: pr.updated_at,
            jira_key,
            pr_role: role.to_string(),
        }
    }
}

/// Extract Jira issue key from PR title or branch name
/// Pattern: PROJECT-123 (uppercase letters followed by dash and numbers)
pub fn extract_jira_key(title: &str, branch: &str) -> Option<String> {
    use regex::Regex;

    // Pattern for Jira keys: 1+ uppercase letters, dash, 1+ digits
    let re = Regex::new(r"([A-Z][A-Z0-9]+-\d+)").ok()?;

    // Try title first
    if let Some(cap) = re.captures(title) {
        return Some(cap[1].to_string());
    }

    // Try branch name (case insensitive for branch)
    let re_ci = Regex::new(r"(?i)([A-Z][A-Z0-9]+-\d+)").ok()?;
    if let Some(cap) = re_ci.captures(branch) {
        // Return uppercase version
        return Some(cap[1].to_uppercase());
    }

    None
}

/// GitHub account info for connection test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccountInfo {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}
