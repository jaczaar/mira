use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeInfo {
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamEvent {
    pub session_id: String,
    pub event_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub status: String,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDiff {
    pub files: Vec<DiffFile>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRResult {
    pub url: String,
    pub number: i64,
    pub branch: String,
}
