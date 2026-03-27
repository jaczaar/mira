use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("Claude CLI not found: {0}")]
    NotInstalled(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Process error: {0}")]
    ProcessError(String),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("GitHub error: {0}")]
    GitHubError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

impl serde::Serialize for ClaudeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
