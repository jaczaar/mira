use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("Node.js download failed: {0}")]
    NodeDownloadError(String),
    #[error("npm error: {0}")]
    NpmError(String),
    #[error("Vite error: {0}")]
    ViteError(String),
}

impl serde::Serialize for WorkspaceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
