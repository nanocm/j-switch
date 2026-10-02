use thiserror::Error;

#[derive(Error, Debug)]
pub enum JdkError {
    #[error("JDK version {0} not found")]
    JdkNotFound(String),

    #[error("JDK {0} matches multiple installations. Use one of these IDs: {1}")]
    AmbiguousJdk(String, String),

    #[error("Invalid JDK path: {0}")]
    InvalidPath(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Environment variable error: {0}")]
    EnvError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("No JDK is currently active")]
    NoActiveJdk,

    #[error("Extraction failed: {0}")]
    ExtractionError(String),

    #[error("Package not found for version: {0}")]
    PackageNotFound(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

pub type Result<T> = std::result::Result<T, JdkError>;
