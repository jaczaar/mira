use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoogleError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("HTTP request failed: {0}")]
    Request(String),
    #[error("Google API error: {status} - {message}")]
    Api { status: u16, message: String },
    #[error("Failed to parse response: {0}")]
    Parse(String),
    #[error("Token error: {0}")]
    Token(String),
    #[error("I/O error: {0}")]
    Io(String),
}

impl serde::Serialize for GoogleError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
