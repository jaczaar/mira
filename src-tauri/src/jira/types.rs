use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraSearchResponse {
    pub expand: Option<String>,
    #[serde(rename = "startAt", default)]
    pub start_at: Option<i32>,
    #[serde(rename = "maxResults", default)]
    pub max_results: Option<i32>,
    #[serde(default)]
    pub total: Option<i32>,
    pub issues: Vec<JiraIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: String,
    pub fields: JiraIssueFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueFields {
    pub summary: String,
    pub description: Option<serde_json::Value>,
    pub status: JiraStatus,
    pub priority: Option<JiraPriority>,
    pub project: JiraProject,
    pub assignee: Option<JiraUser>,
    pub reporter: Option<JiraUser>,
    #[serde(rename = "timeoriginalestimate")]
    pub time_original_estimate: Option<i64>,
    #[serde(rename = "timeestimate")]
    pub time_estimate: Option<i64>,
    #[serde(rename = "timespent")]
    pub time_spent: Option<i64>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub duedate: Option<String>,
    #[serde(rename = "issuetype")]
    pub issue_type: Option<JiraIssueType>,
    pub labels: Option<Vec<String>>,
    pub parent: Option<JiraParent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraParent {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
    pub fields: Option<JiraParentFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraParentFields {
    pub summary: Option<String>,
    pub status: Option<JiraStatus>,
    #[serde(rename = "issuetype")]
    pub issue_type: Option<JiraIssueType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub status_category: Option<JiraStatusCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusCategory {
    pub id: i32,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    pub id: String,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorklog {
    pub id: Option<String>,
    #[serde(rename = "issueId")]
    pub issue_id: Option<String>,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: i64,
    pub started: String,
    pub comment: Option<JiraWorklogComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorklogComment {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: i32,
    pub content: Vec<JiraWorklogContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorklogContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub content: Option<Vec<JiraWorklogTextContent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraWorklogTextContent {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorklogRequest {
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: i64,
    pub started: String,
    pub comment: Option<JiraWorklogComment>,
}

impl CreateWorklogRequest {
    pub fn new(time_spent_seconds: i64, started: DateTime<Utc>, comment: Option<String>) -> Self {
        let comment = comment.map(|text| JiraWorklogComment {
            doc_type: "doc".to_string(),
            version: 1,
            content: vec![JiraWorklogContent {
                content_type: "paragraph".to_string(),
                content: Some(vec![JiraWorklogTextContent {
                    text_type: "text".to_string(),
                    text,
                }]),
            }],
        });

        Self {
            time_spent_seconds,
            started: started.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string(),
            comment,
        }
    }
}

// Simplified issue representation for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleIssue {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub status: String,
    pub status_category: String,
    pub priority: Option<String>,
    pub project_key: String,
    pub project_name: String,
    pub time_estimate_seconds: Option<i64>,
    pub time_spent_seconds: Option<i64>,
    pub due_date: Option<String>,
    pub issue_type: Option<String>,
    pub labels: Vec<String>,
    pub url: String,
    pub parent_key: Option<String>,
    pub parent_summary: Option<String>,
    pub is_epic: bool,
}

impl From<JiraIssue> for SimpleIssue {
    fn from(issue: JiraIssue) -> Self {
        let status_category = issue
            .fields
            .status
            .status_category
            .map(|sc| sc.key)
            .unwrap_or_else(|| "undefined".to_string());

        let is_epic = issue
            .fields
            .issue_type
            .as_ref()
            .map(|it| it.name.to_lowercase() == "epic")
            .unwrap_or(false);

        let parent_key = issue.fields.parent.as_ref().map(|p| p.key.clone());
        let parent_summary = issue
            .fields
            .parent
            .as_ref()
            .and_then(|p| p.fields.as_ref())
            .and_then(|f| f.summary.clone());

        Self {
            id: issue.id,
            key: issue.key.clone(),
            summary: issue.fields.summary,
            status: issue.fields.status.name,
            status_category,
            priority: issue.fields.priority.map(|p| p.name),
            project_key: issue.fields.project.key,
            project_name: issue.fields.project.name,
            time_estimate_seconds: issue.fields.time_estimate,
            time_spent_seconds: issue.fields.time_spent,
            due_date: issue.fields.duedate,
            issue_type: issue.fields.issue_type.map(|it| it.name),
            labels: issue.fields.labels.unwrap_or_default(),
            url: {
                // self_url is like: https://org.atlassian.net/rest/api/3/issue/12345
                // We need: https://org.atlassian.net/browse/PROJ-123
                let base_url = issue.self_url
                    .split("/rest/api/")
                    .next()
                    .unwrap_or(&issue.self_url);
                format!("{}/browse/{}", base_url, issue.key)
            },
            parent_key,
            parent_summary,
            is_epic,
        }
    }
}
