use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::process::Child;

pub struct SessionInfo {
    pub session_id: String,
    pub repo_path: PathBuf,
    pub active_process: Option<Child>,
}

#[derive(Default)]
pub struct ChatState {
    pub sessions: Mutex<HashMap<String, SessionInfo>>,
}
